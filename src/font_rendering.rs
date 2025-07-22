#![allow(dead_code)]

//! This module handles font texture atlasing, and texture layout

use crate::gl_primitives::{compile_shader, link_shader};
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::error::GlyphLoadingError;
use font_kit::font::Font as FKFont;
use font_kit::hinting::HintingOptions;
use gl::types::{GLint, GLuint};
use harfbuzz_rs::{Feature, Font as HBFont, Owned, UnicodeBuffer};
use pathfinder_color::ColorF;
use pathfinder_geometry::rect::{RectF, RectI};
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{Vector2F, Vector2I, vec2f, vec2i};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::mem::offset_of;
use std::sync::{Arc, Weak};
// TODO: shrink canvas
// TODO: resize canvas

// The layout of canvas
//
// There are several lines with height of 'line_height' that are not tall in glyph height (called 'short glyphs')
//
// There are few lines for glyphs with height larger than `line_height` (called 'tall glyphs')
//
// The height of each line and width of each glyphs are always multiple of 'mip_cell_size' to avoid
// problems with mipmapping.
//
//(0,0)
//  *--------------------------------+
//  | |     |  |  |  |  | ....       |   shot glyph line
//  | |     |  |  |  |  | ....       |
//  +--------------------------------+
//  | |   |  |  |  |    | ....       |   shot glyph line
//  | |   |  |  |  |    | ....       |
//  +-------------------*------------+
//  |                   ^            |   unused area
//  |           short_glyph_cursor   |
//  +--------------------------------+  ------------------------
//  |   | |                          |                       ^
//  |   | |                          |    tall glyph line    |  tall_glyph_line_total_height
//  |   | |                          |                       V
//  +-----*--------------------------+  -------------------------
//        ^ tall_glyph_cursor

#[derive(Clone)]
pub struct FontAtlas {
    /// The canvas we have drawn glyphs.
    /// When we have too many characters to use, we might need multiple canvas to fit all characters
    canvases: Vec<CanvasClone>,

    // static information of the glyphs / canvas
    /// The size of text in em units
    font_em_size: f32,
    /// The height of 'short glyphs' line
    short_line_height: u32,
    /// The maximum texture size. This is limited by GPU or graphics API
    #[allow(unused)]
    max_texture_size: u32,
    /// The minimum 'unit' of line height or glyph width.
    /// This is needed to avoid problems with mipmapping.
    mip_cell_size: u32,
    /// The size of padding in the glyphs.
    /// Currently same as mip_cell_size, but in the future I may implement option to
    /// disable padding when user doesn't consider magnification.
    pad_size: i32,

    // The state of the canvas rendering
    canvas_state: CanvasState,

    /// The location information of the rendered glyphs
    glyphs: HashMap<GlyphId, GlyphInfo>,
}

#[repr(transparent)]
struct CanvasClone(Canvas);

impl CanvasClone {
    pub fn new(size: Vector2I, format: Format) -> Self {
        CanvasClone(Canvas::new(size, format))
    }
}

impl Clone for CanvasClone {
    fn clone(&self) -> Self {
        Self(Canvas {
            pixels: self.0.pixels.clone(),
            size: self.0.size,
            stride: self.0.stride,
            format: self.0.format,
        })
    }
}

#[derive(Clone)]
struct CanvasState {
    // current canvas (= canvases.last()) rendering positon information
    /// The current position of short glyph line.
    /// This value is at bottom left corner of cursor
    short_glyph_cursor: Vector2I,
    /// The current position of tall glyph line.
    /// This value is at bottom left corner of cursor
    tall_glyph_cursor: Vector2I,
    /// The topmost position of the 'tall glyphs'
    tall_glyph_line_min_y: i32,
    /// The size of the canvas
    canvas_size: Vector2I,
}

impl CanvasState {
    fn new(line_height: i32, canvas_size: Vector2I) -> Self {
        Self {
            short_glyph_cursor: vec2i(0, line_height),
            tall_glyph_cursor: vec2i(0, canvas_size.y()),
            tall_glyph_line_min_y: canvas_size.y(),
            canvas_size,
        }
    }
}

#[derive(Clone)]
struct GlyphId(Weak<FKFont>, u32);

impl PartialEq<Self> for GlyphId {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0) && self.1 == other.1
    }
}

impl Eq for GlyphId {}

impl Hash for GlyphId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
        self.1.hash(state);
    }
}

/// The information of each glyph.
///
/// This information contains:
/// - The rect of glyph and advance in em-unit (1.0 is 1em)
/// - The rect of glyph in the atlas texture and the index of atlas texture
#[derive(Default, Copy, Clone, Debug)]
#[non_exhaustive]
pub struct GlyphInfo {
    pub canvas_id: usize,
    #[allow(dead_code)]
    pub glyph_id: u32,
    pub advance: Vector2F,
    pub rasterize_offset: Vector2F,
    pub rasterize_size: Vector2F,
    pub atlas_origin: Vector2I,
    pub atlas_size: Vector2I,
}

impl FontAtlas {
    pub fn new(font_em_size: f32, max_texture_size: u32, mip_cell_size: u32) -> FontAtlas {
        assert!(font_em_size > 0.0);
        assert!(max_texture_size > font_em_size.ceil() as u32);
        assert!(max_texture_size > 0 && max_texture_size.is_power_of_two());
        assert!(mip_cell_size > 0 && mip_cell_size.is_power_of_two());

        let short_line_height = (font_em_size.ceil() as u32).next_multiple_of(mip_cell_size);
        let expected_lines = 20; // TODO: configurable
        let size = (expected_lines * short_line_height)
            .next_power_of_two()
            .min(max_texture_size);
        let canvas = CanvasClone::new(Vector2I::splat(size as i32), Format::A8);
        Self {
            font_em_size,
            short_line_height,
            max_texture_size,
            mip_cell_size,
            pad_size: mip_cell_size as i32,

            canvas_state: CanvasState::new(short_line_height as i32, canvas.0.size),
            canvases: vec![canvas],
            glyphs: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn font_em_size(&self) -> f32 {
        self.font_em_size
    }

    pub fn canvas_size(&self) -> Vector2I {
        self.canvas_state.canvas_size
    }

    pub fn canvases(&self) -> &[Canvas] {
        unsafe {
            // SAFETY: CloneCanvas is transparent to Canvas
            std::slice::from_raw_parts(self.canvases.as_ptr().cast(), self.canvases.len())
        }
    }

    /// Prepares glyphs and returns list of UV location
    pub fn get_glyphs(
        &mut self,
        glyphs: &[(Arc<FKFont>, u32)],
    ) -> Result<(Vec<GlyphInfo>, Vec<(Arc<FKFont>, u32)>), GlyphLoadingError> {
        let mut result = vec![GlyphInfo::default(); glyphs.len()];
        let mut glyphs_to_add = HashSet::new();

        for (i, &(ref font, glyph_id)) in glyphs.iter().enumerate() {
            let id = GlyphId(Arc::downgrade(font), glyph_id);
            if let Some(&info) = self.glyphs.get(&id) {
                result[i] = info;
            } else {
                let raster_scale = self.font_em_size / font.metrics().units_per_em as f32;
                let typographic_bounds = font.typographic_bounds(glyph_id)?;
                let rasterize_bounds = (typographic_bounds * raster_scale).round_out().to_i32();
                let rasterize_offset = vec2i(
                    -rasterize_bounds.origin().x(),
                    rasterize_bounds.origin().y(),
                );
                let rasterize_size = rasterize_bounds.size();
                let rasterize_size = vec2i(
                    (rasterize_size.x() as u32).next_multiple_of(self.mip_cell_size) as i32,
                    (rasterize_size.y() as u32).next_multiple_of(self.mip_cell_size) as i32,
                );
                let advance = font.advance(glyph_id)? * raster_scale;

                result[i] = GlyphInfo {
                    canvas_id: 0,
                    glyph_id,
                    advance: advance / self.font_em_size,
                    rasterize_offset: rasterize_offset.to_f32() / self.font_em_size,
                    rasterize_size: rasterize_size.to_f32() / self.font_em_size,
                    atlas_origin: Vector2I::zero(),
                    atlas_size: Vector2I::zero(),
                };
                glyphs_to_add.insert(id);
            }
        }

        let glyphs_to_add = glyphs_to_add
            .into_iter()
            .map(|GlyphId(font, glyph)| (Weak::upgrade(&font).unwrap(), glyph))
            .collect();
        Ok((result, glyphs_to_add))
    }

    pub fn rasterize_glyphs(
        &mut self,
        glyphs_to_add: &[(Arc<FKFont>, u32)],
    ) -> Result<bool, GlyphLoadingError> {
        {
            let hinting = HintingOptions::None;
            let options = RasterizationOptions::GrayscaleAa;

            // process glyphs not added to this atlas
            let mut short_rasterize_information = vec![];
            let mut tall_rasterize_information = vec![];

            struct RasterizeInformation<'a> {
                font: &'a Arc<FKFont>,
                glyph_id: u32,
                rasterize_offset: Vector2I,
                rasterize_size: Vector2I,
                advance: Vector2F,
                // canvas id, position, offset for raster
                rasterize_position: Option<(usize, Vector2I)>,
            }

            for &(ref font, glyph_id) in glyphs_to_add {
                let id = GlyphId(Arc::downgrade(&font), glyph_id);
                if self.glyphs.contains_key(&id) {
                    continue;
                }

                let raster_scale = self.font_em_size / font.metrics().units_per_em as f32;
                let typographic_bounds = font.typographic_bounds(glyph_id)?;
                let rasterize_bounds = (typographic_bounds * raster_scale).round_out().to_i32();
                let rasterize_offset = vec2i(
                    -rasterize_bounds.origin().x(),
                    rasterize_bounds.origin().y(),
                );
                let rasterize_size = rasterize_bounds.size();
                let rasterize_size = vec2i(
                    (rasterize_size.x() as u32).next_multiple_of(self.mip_cell_size) as i32,
                    (rasterize_size.y() as u32).next_multiple_of(self.mip_cell_size) as i32,
                );
                let is_short = rasterize_size.y() as u32 <= self.short_line_height;
                let advance = font.advance(glyph_id)? * raster_scale;

                if rasterize_size.x() >= self.canvas_state.canvas_size.x()
                    || rasterize_size.y() >= self.canvas_state.canvas_size.y()
                {
                    panic!("Rasterized text size is too big to fit in the canvas size.")
                }

                let rasterize_information_ref = if is_short {
                    &mut short_rasterize_information
                } else {
                    &mut tall_rasterize_information
                };

                rasterize_information_ref.push(RasterizeInformation {
                    font,
                    glyph_id,
                    rasterize_offset,
                    rasterize_size,
                    advance,
                    rasterize_position: None,
                })
            }

            if short_rasterize_information.is_empty() && tall_rasterize_information.is_empty() {
                return Ok(false);
            }

            // We try to layout by wider to shorter. I hope this should reduce unused space
            short_rasterize_information.sort_by_key(|f| Reverse(f.rasterize_size.x()));
            tall_rasterize_information.sort_by_key(|f| Reverse(f.rasterize_size.x()));

            let mut canvas_index = self.canvases.len() - 1;
            let mut canvas_state = self.canvas_state.clone();
            // We do layout-ing and rasterize-ing in different pass for future resize support

            // layout 'short glyphs' line
            {
                while {
                    let mut needs_next_line = false;
                    for information in &mut short_rasterize_information {
                        if information.rasterize_position.is_none() {
                            if canvas_state.short_glyph_cursor.x() + information.rasterize_size.x()
                                < canvas_state.canvas_size.x()
                            {
                                // There's space for this glyph in current glyph line so add to this line
                                information.rasterize_position =
                                    Some((canvas_index, canvas_state.short_glyph_cursor));
                                canvas_state.short_glyph_cursor +=
                                    vec2i(information.rasterize_size.x() + self.pad_size, 0);
                            } else {
                                needs_next_line = true;
                            }
                        }
                    }
                    needs_next_line
                } {
                    // We have to move to next line
                    canvas_state.short_glyph_cursor = vec2i(
                        0,
                        canvas_state.short_glyph_cursor.y()
                            + self.short_line_height as i32
                            + self.pad_size,
                    );
                    if canvas_state.short_glyph_cursor.y() > self.canvas_state.tall_glyph_line_min_y
                    {
                        // We don't have space for new line in this canvas so we create new canvas.
                        // TODO: We should resize the canvas size if current canvas is the first canvas in the row.
                        self.canvases
                            .push(CanvasClone::new(self.canvas_state.canvas_size, Format::A8));
                        canvas_index += 1;
                        canvas_state = CanvasState::new(
                            self.short_line_height as i32,
                            canvas_state.canvas_size,
                        );
                    }
                }
            }

            // layout 'tall glyphs' line
            {
                while {
                    let mut needs_next_line = false;
                    for information in &mut tall_rasterize_information {
                        if information.rasterize_position.is_none() {
                            let min_y = canvas_state.tall_glyph_cursor.y()
                                - information.rasterize_size.y()
                                - self.pad_size as i32;

                            if canvas_state.tall_glyph_cursor.x() + information.rasterize_size.x()
                                < canvas_state.canvas_size.x()
                                && min_y >= canvas_state.short_glyph_cursor.y()
                            {
                                // There's space for this glyph in current glyph line so add to this line
                                information.rasterize_position =
                                    Some((canvas_index, canvas_state.tall_glyph_cursor));
                                canvas_state.tall_glyph_cursor +=
                                    vec2i(information.rasterize_size.x() + self.pad_size as i32, 0);
                                canvas_state.tall_glyph_line_min_y =
                                    canvas_state.tall_glyph_line_min_y.min(min_y);
                            } else {
                                needs_next_line = true;
                            }
                        }
                    }
                    needs_next_line
                } {
                    // We have to move to next line
                    if canvas_state.tall_glyph_line_min_y != canvas_state.tall_glyph_cursor.y() {
                        canvas_state.tall_glyph_cursor =
                            vec2i(0, canvas_state.tall_glyph_line_min_y - self.pad_size as i32);
                    } else {
                        // This means we couldn't insert no characters to the last line due to height problem
                        // so we should move to next canvas
                        // TODO: We should resize the canvas size if current canvas is the first canvas in the row.
                        self.canvases
                            .push(CanvasClone::new(self.canvas_state.canvas_size, Format::A8));
                        canvas_index += 1;
                        canvas_state = CanvasState::new(
                            self.short_line_height as i32,
                            canvas_state.canvas_size,
                        );
                    }
                }
            }

            self.canvas_state = canvas_state;

            // We successfully finished layout so we next rasterize glyphs.
            for information in short_rasterize_information
                .iter()
                .chain(tall_rasterize_information.iter())
            {
                let (canvas_id, rasterize_position) = information.rasterize_position.unwrap();

                information.font.rasterize_glyph(
                    &mut self.canvases[canvas_id].0,
                    information.glyph_id,
                    self.font_em_size,
                    Transform2F::from_translation(
                        (rasterize_position + information.rasterize_offset).to_f32(),
                    ),
                    hinting,
                    options,
                )?;

                let id = GlyphId(Arc::downgrade(&information.font), information.glyph_id);

                self.glyphs.insert(
                    id,
                    GlyphInfo {
                        canvas_id,
                        glyph_id: information.glyph_id,
                        advance: information.advance / self.font_em_size,
                        rasterize_offset: information.rasterize_offset.to_f32() / self.font_em_size,
                        rasterize_size: information.rasterize_size.to_f32() / self.font_em_size,
                        atlas_origin: rasterize_position,
                        atlas_size: information.rasterize_size,
                    },
                );
            }
        }

        Ok(true)
    }

    /// Prepares glyphs and returns list of UV location
    pub fn prepare_glyphs(
        &mut self,
        glyphs: &[(Arc<FKFont>, u32)],
    ) -> Result<(Vec<GlyphInfo>, bool), GlyphLoadingError> {
        let (glyph_infos, glyphs_to_add) = self.get_glyphs(glyphs)?;

        if glyphs_to_add.is_empty() {
            return Ok((glyph_infos, false));
        }

        self.rasterize_glyphs(&glyphs_to_add)?;

        let (glyph_infos, glyphs_to_add) = self.get_glyphs(&glyphs)?;
        assert!(glyphs_to_add.is_empty());

        Ok((glyph_infos, true))
    }
}

pub struct FontRenderer {
    shader_program: GLuint,
    points_vbo: GLuint,
    points_vao: GLuint,
    font_textures_attrib: GLint,
    font_color_attrib: GLint,
    font_atlas_texture: GLuint,
    font_atlas_texture_size: Vector2I,
    font_atlas_texture_dimension: usize,
}

// attributes definition
#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct PointInfo {
    pos: [f32; 2],
    uv: [f32; 2],
    tex: f32,
}

const _: () = {
    ["uv and tex must be in a row"][offset_of!(PointInfo, uv) + 8 - offset_of!(PointInfo, tex)];
};

impl FontRenderer {
    pub fn new() -> Self {
        unsafe {
            // shader
            let vs = compile_shader(
                gl::VERTEX_SHADER,
                "#version 400\n\
                layout(location = 0) in vec2 in_pos;\n\
                layout(location = 1) in vec3 in_uv_tex;\n\
                out vec3 v2f_uv_tex;\n\
                void main() {\n\
                    v2f_uv_tex = in_uv_tex;\n\
                    gl_Position.xy = in_pos;\n\
                    gl_Position.zw = vec2(0, 1);\n\
                }\n",
            );
            let fs = compile_shader(
                gl::FRAGMENT_SHADER,
                "#version 400\n\
                \n\
                in vec3 v2f_uv_tex;\n\
                out vec4 color;\n\
                \n\
                uniform sampler2DArray font_textures;\n\
                uniform vec4 font_color;\n\
                \n\
                void main() {\n\
                    color.rgb = font_color.rgb;\n\
                    color.a = texture(font_textures, v2f_uv_tex.xyz).r * font_color.a;\n\
                }\n",
            );
            let shader_program = link_shader(&[fs, vs]);
            let in_pos_attrib = gl::GetAttribLocation(shader_program, c"in_pos".as_ptr()) as GLuint;
            let in_uv_tex_attrib =
                gl::GetAttribLocation(shader_program, c"in_uv_tex".as_ptr()) as GLuint;
            let font_textures_attrib =
                gl::GetUniformLocation(shader_program, c"font_textures".as_ptr());
            let font_color_attrib = gl::GetUniformLocation(shader_program, c"font_color".as_ptr());
            assert!(in_pos_attrib != -1 as GLint as GLuint, "in_pos not found");
            assert!(
                in_uv_tex_attrib != -1 as GLint as GLuint,
                "in_uv_tex not found"
            );
            assert!(font_textures_attrib != -1, "font_textures not found");
            assert!(font_color_attrib != -1, "font_textures not found");

            let mut points_vbo = 0;
            gl::GenBuffers(1, &mut points_vbo);

            let mut points_vao = 0;
            gl::GenVertexArrays(1, &mut points_vao);
            gl::BindVertexArray(points_vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
            gl::EnableVertexAttribArray(in_pos_attrib as _);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<PointInfo>() as _,
                std::ptr::without_provenance(offset_of!(PointInfo, pos)),
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
            gl::EnableVertexAttribArray(in_uv_tex_attrib as _);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<PointInfo>() as _,
                std::ptr::without_provenance(offset_of!(PointInfo, uv)),
            );

            Self {
                shader_program,
                points_vbo,
                points_vao,
                font_textures_attrib,
                font_color_attrib,
                font_atlas_texture: 0,
                font_atlas_texture_size: Vector2I::zero(),
                font_atlas_texture_dimension: 0,
            }
        }
    }

    fn alloc_texture(&mut self, atlas: &FontAtlas) {
        //println!("alloc_texture");
        unsafe {
            if self.font_atlas_texture != 0 {
                gl::DeleteTextures(1, &self.font_atlas_texture);
            }
            gl::GenTextures(1, &mut self.font_atlas_texture);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.font_atlas_texture);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexStorage3D(
                gl::TEXTURE_2D_ARRAY,
                1,
                gl::R8 as _,
                atlas.canvas_size().x() as _,
                atlas.canvas_size().y() as _,
                atlas.canvases().len() as _,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as _,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as _,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as _,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as _,
            );
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);

            self.font_atlas_texture_size = atlas.canvas_size();
            self.font_atlas_texture_dimension = atlas.canvases().len();
        }
    }

    /// Updates and uploads font atlas texture this font renderer uses
    pub fn update_texture(&mut self, atlas: &FontAtlas) {
        unsafe {
            if self.font_atlas_texture_size != atlas.canvas_size()
                || self.font_atlas_texture_dimension != atlas.canvases().len()
            {
                self.alloc_texture(atlas);
            }
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.font_atlas_texture);

            for (index, canvas) in atlas.canvases().iter().enumerate() {
                gl::TexSubImage3D(
                    gl::TEXTURE_2D_ARRAY,
                    0,
                    0,
                    0,
                    index as _,
                    atlas.canvas_size().x() as _,
                    atlas.canvas_size().y() as _,
                    1,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    canvas.pixels.as_ptr().cast(),
                );
            }
            gl::GenerateMipmap(gl::TEXTURE_2D_ARRAY);
        }
    }

    fn generate_points(
        &self,
        glyphs: impl IntoIterator<Item = (GlyphInfo, Transform2F)>,
    ) -> Vec<PointInfo> {
        let glyphs = glyphs.into_iter();
        let uv_scale = Vector2F::splat(1.0) / self.font_atlas_texture_size.to_f32();
        let mut points = Vec::<PointInfo>::with_capacity(glyphs.size_hint().0 * 6);
        for (info, transform) in glyphs {
            let poly_rect = RectF::new(
                info.rasterize_offset * vec2f(-1.0, 1.0),
                info.rasterize_size,
            );
            let uv_rect =
                RectI::new(info.atlas_origin, info.atlas_size * vec2i(1, -1)).to_f32() * uv_scale;

            fn as_array(a: Vector2F) -> [f32; 2] {
                [a.x(), a.y()]
            }

            macro_rules! point {
                ($f: ident) => {
                    PointInfo {
                        pos: as_array(transform * poly_rect.$f()),
                        uv: as_array(uv_rect.$f()),
                        tex: info.canvas_id as f32,
                    }
                };
            }

            let origin = point!(origin);
            let lower_left = point!(lower_left);
            let upper_right = point!(upper_right);
            let lower_right = point!(lower_right);

            points.push(upper_right);
            points.push(lower_left);
            points.push(origin);
            points.push(lower_left);
            points.push(upper_right);
            points.push(lower_right);
        }
        points
    }

    /// Renders glyphs in specified color.
    pub fn draw_glyphs(
        &self,
        color: ColorF,
        glyphs: impl IntoIterator<Item = (GlyphInfo, Transform2F)>,
    ) {
        // prepare buffer
        inner(self, color, &self.generate_points(glyphs));

        fn inner(this: &FontRenderer, color: ColorF, points: &[PointInfo]) {
            // update VBO
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, this.points_vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    size_of_val::<[_]>(points) as isize,
                    points.as_ptr().cast(),
                    gl::STATIC_DRAW,
                );
            }

            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl::UseProgram(this.shader_program);

                gl::BindVertexArray(this.points_vao);

                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D_ARRAY, this.font_atlas_texture);
                gl::Uniform1i(this.font_textures_attrib, 0);

                gl::Uniform4f(
                    this.font_color_attrib,
                    color.r(),
                    color.g(),
                    color.b(),
                    color.a(),
                );

                gl::DrawArrays(gl::TRIANGLES, 0, points.len() as i32);
                gl::Disable(gl::BLEND);
            }
        }
    }

    /// Renders text with extremely simple text layout algorithm: select glyph by one character and
    /// place glyphs
    #[allow(dead_code)]
    pub fn draw_text_simple(
        &mut self,
        atlas: &mut FontAtlas,
        font: Arc<FKFont>,
        color: ColorF,
        transform: Transform2F,
        text: &str,
    ) -> Result<(), GlyphLoadingError> {
        let glyphs = text
            .chars()
            .map(|c| font.glyph_for_char(c).ok_or(GlyphLoadingError::NoSuchGlyph))
            .collect::<Result<Vec<_>, _>>()?;
        let (glyph_info, update) = atlas.prepare_glyphs(
            &glyphs
                .iter()
                .map(|&g| (Arc::clone(&font), g))
                .collect::<Vec<_>>(),
        )?;
        if update {
            self.update_texture(atlas);
        }

        let matrix = transform.matrix;

        let mut cursor = transform.vector;
        self.draw_glyphs(
            color,
            glyph_info.iter().map(|&info| {
                let advance = matrix * info.advance;
                let transform = Transform2F {
                    matrix,
                    vector: cursor,
                };
                cursor += advance;
                (info, transform)
            }),
        );

        Ok(())
    }
}

pub struct TextArranger {
    fonts: Vec<(Arc<Owned<HBFont<'static>>>, Arc<FKFont>)>,
}

impl TextArranger {
    pub fn new(
        handles: impl IntoIterator<Item = font_kit::handle::Handle>,
    ) -> Result<Self, font_kit::error::FontLoadingError> {
        let handles = handles.into_iter();
        let mut fonts = Vec::with_capacity(handles.size_hint().0);
        for handle in handles {
            let (harfbuzz, font_kit) = loader::load_font(&handle)?;
            fonts.push((Arc::new(HBFont::new(harfbuzz)), Arc::new(font_kit)));
        }
        Ok(Self { fonts })
    }

    pub fn layout(&self, mut text: &str, features: &[Feature]) -> Layout {
        if text.is_empty() {
            return Layout {
                transform: Transform2F::default(),
                glyphs: vec![],
                transforms: vec![],
                advance: Vector2F::zero(),
            };
        }
        let chars = text.chars().count();
        let mut glyphs = Vec::with_capacity(chars);
        let mut transforms = Vec::with_capacity(chars);

        let mut cursor = vec2f(0., 0.);

        let mut buffer = UnicodeBuffer::new().add_str(text);

        let mut font_index = 0;
        let mut tried_with_primary_font = false;
        'font_loop: loop {
            let glyph_buffer = harfbuzz_rs::shape(&self.fonts[font_index].0, buffer, features);

            let glyph_infos = glyph_buffer.get_glyph_infos();
            let positions = glyph_buffer.get_glyph_positions();

            let scale = 1. / self.fonts[font_index].0.scale().0 as f32;

            for glyph_index in 0..glyph_buffer.len() {
                let glyph = &glyph_infos[glyph_index];
                let glyph_position = &positions[glyph_index];
                let glyph_id = glyph.codepoint;
                let offset = Vector2I::new(glyph_position.x_offset, glyph_position.y_offset)
                    .to_f32()
                    * scale;
                let advance = Vector2I::new(glyph_position.x_advance, glyph_position.y_advance)
                    .to_f32()
                    * scale;

                if glyph_id == 0 {
                    if !tried_with_primary_font || font_index != 0 {
                        text = &text[glyph.cluster as usize..];
                        //println!("fall-backing {} with {}", text.chars().next().unwrap(), font_index);

                        if font_index == 0 {
                            tried_with_primary_font = true;
                        }

                        buffer = UnicodeBuffer::new().add_str(text);
                        font_index += 1;
                        if font_index == self.fonts.len() {
                            font_index = 0;
                        }
                        continue 'font_loop;
                    } else {
                        //println!("fall-backing {} with all fonts failed.", text.chars().next().unwrap());
                        tried_with_primary_font = false;
                    }
                } else {
                    tried_with_primary_font = false;
                }

                let transform = Transform2F::from_translation(cursor + offset);

                glyphs.push((Arc::clone(&self.fonts[font_index].1), glyph_id));
                transforms.push(transform);

                cursor += advance;
            }
            break;
        }

        Layout {
            transform: Transform2F::default(),
            glyphs,
            transforms,
            advance: cursor,
        }
    }

    pub fn metrics(&self) -> FontMetrics {
        let metrics = self.fonts[0].1.metrics();
        let y_scale = 1.0 / metrics.units_per_em as f32;
        FontMetrics {
            underline_position: metrics.underline_position * y_scale,
            underline_thickness: metrics.underline_thickness * y_scale,
            line_gap: metrics.line_gap * y_scale,
            cap_height: metrics.cap_height * y_scale,
            x_height: metrics.x_height * y_scale,
        }
    }
}

pub struct Layout {
    transform: Transform2F,
    glyphs: Vec<(Arc<FKFont>, u32)>,
    transforms: Vec<Transform2F>,
    advance: Vector2F,
}

impl Layout {
    pub fn apply_transform(&mut self, transform: Transform2F) {
        self.transform = transform * self.transform;
        for t in &mut self.transforms {
            *t = transform * *t;
        }
        self.advance = transform.matrix * self.advance;
    }

    pub fn transform(&self) -> Transform2F {
        self.transform
    }

    pub fn glyphs(&self) -> &[(Arc<FKFont>, u32)] {
        &self.glyphs
    }

    pub fn transforms(&self) -> &[Transform2F] {
        &self.transforms
    }

    pub fn cursor_advance(&self) -> Vector2F {
        self.advance
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub struct FontMetrics {
    pub underline_position: f32,
    pub underline_thickness: f32,
    pub line_gap: f32,
    pub cap_height: f32,
    pub x_height: f32,
}

mod loader {
    use font_kit::canvas::{Canvas, RasterizationOptions};
    use font_kit::error::{FontLoadingError, GlyphLoadingError};
    use font_kit::file_type::FileType;
    use font_kit::font::Font as FKFont;
    use font_kit::handle::Handle;
    use font_kit::hinting::HintingOptions;
    use font_kit::loader::{FallbackResult, Loader};
    use font_kit::metrics::Metrics;
    use font_kit::outline::OutlineSink;
    use font_kit::properties::Properties;
    use harfbuzz_rs::{Face, Owned};
    use std::fs::File;
    use std::io::Read;
    use std::rc::Rc;
    use std::sync::Arc;

    pub(super) fn load_font(
        handle: &Handle,
    ) -> Result<(Owned<Face<'static>>, FKFont), FontLoadingError> {
        let loader = HarfbuzzLoader::from_handle(handle)?;
        let harfbuzz = Rc::try_unwrap(loader.0).unwrap();
        let font = loader.1;
        Ok((harfbuzz, font))
    }

    #[derive(Clone)]
    struct HarfbuzzLoader(Rc<Owned<Face<'static>>>, FKFont);

    impl Loader for HarfbuzzLoader {
        type NativeFont = ();

        fn from_bytes(font_data: Arc<Vec<u8>>, font_index: u32) -> Result<Self, FontLoadingError> {
            let harfbuzz = Rc::new(Face::<'static>::new(Vec::clone(&font_data), font_index));
            let font_kit = FKFont::from_bytes(font_data, font_index)?;
            Ok(HarfbuzzLoader(harfbuzz, font_kit))
        }

        fn from_file(file: &mut File, font_index: u32) -> Result<Self, FontLoadingError> {
            let mut font_data = vec![];
            file.read_to_end(&mut font_data)?;
            let harfbuzz = Rc::new(Face::<'static>::new(Vec::clone(&font_data), font_index));
            let font_kit = FKFont::from_bytes(font_data.into(), font_index)?;
            Ok(HarfbuzzLoader(harfbuzz, font_kit))
        }

        // region unimplemented
        unsafe fn from_native_font(_: Self::NativeFont) -> Self {
            unimplemented!()
        }

        fn analyze_bytes(_: Arc<Vec<u8>>) -> Result<FileType, FontLoadingError> {
            unimplemented!()
        }

        fn analyze_file(_: &mut File) -> Result<FileType, FontLoadingError> {
            unimplemented!()
        }

        fn native_font(&self) -> Self::NativeFont {
            unimplemented!()
        }

        fn postscript_name(&self) -> Option<String> {
            unimplemented!()
        }

        fn full_name(&self) -> String {
            unimplemented!()
        }

        fn family_name(&self) -> String {
            unimplemented!()
        }

        fn is_monospace(&self) -> bool {
            unimplemented!()
        }

        fn properties(&self) -> Properties {
            unimplemented!()
        }

        fn glyph_count(&self) -> u32 {
            unimplemented!()
        }

        fn glyph_for_char(&self, _: char) -> Option<u32> {
            unimplemented!()
        }

        fn outline<S>(&self, _: u32, _: HintingOptions, _: &mut S) -> Result<(), GlyphLoadingError>
        where
            S: OutlineSink,
        {
            unimplemented!()
        }

        fn typographic_bounds(
            &self,
            _: u32,
        ) -> Result<pathfinder_geometry::rect::RectF, GlyphLoadingError> {
            unimplemented!()
        }

        fn advance(
            &self,
            _: u32,
        ) -> Result<pathfinder_geometry::vector::Vector2F, GlyphLoadingError> {
            unimplemented!()
        }

        fn origin(
            &self,
            _: u32,
        ) -> Result<pathfinder_geometry::vector::Vector2F, GlyphLoadingError> {
            unimplemented!()
        }

        fn metrics(&self) -> Metrics {
            todo!()
        }

        fn copy_font_data(&self) -> Option<Arc<Vec<u8>>> {
            unimplemented!()
        }

        fn supports_hinting_options(&self, _: HintingOptions, _: bool) -> bool {
            unimplemented!()
        }

        fn rasterize_glyph(
            &self,
            _: &mut Canvas,
            _: u32,
            _: f32,
            _: pathfinder_geometry::transform2d::Transform2F,
            _: HintingOptions,
            _: RasterizationOptions,
        ) -> Result<(), GlyphLoadingError> {
            unimplemented!()
        }

        fn get_fallbacks(&self, _: &str, _: &str) -> FallbackResult<Self> {
            unimplemented!()
        }

        fn load_font_table(&self, _: u32) -> Option<Box<[u8]>> {
            unimplemented!()
        }
        // endregion
    }
}
