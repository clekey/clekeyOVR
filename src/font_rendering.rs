//! This module handles font texture atlasing, and texture layout

use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::error::GlyphLoadingError;
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use gl::types::{GLenum, GLint, GLsizei, GLuint};
use pathfinder_color::ColorF;
use pathfinder_geometry::rect::RectI;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{Vector2F, Vector2I, vec2i};
use std::cmp::Reverse;
use std::collections::HashMap;
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

pub struct FontAtlas {
    /// The canvas we have drawn glyphs.
    /// When we have too many characters to use, we might need multiple canvas to fit all characters
    canvases: Vec<Canvas>,

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

    // The state of the canvas rendering
    canvas_state: CanvasState,

    /// The location information of the rendered glyphs
    glyphs: HashMap<GlyphId, GlyphInfo>,
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

struct GlyphId(Weak<Font>, u32);

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

#[derive(Default, Copy, Clone, Debug)]
#[non_exhaustive]
pub struct GlyphInfo {
    pub canvas_id: usize,
    pub glyph_id: u32,
    pub advance: Vector2F,
    pub rasterize_offset: Vector2I,
    pub glyph_origin: Vector2I,
    pub glyph_size: Vector2I,
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
        let canvas = Canvas::new(Vector2I::splat(size as i32), Format::A8);
        Self {
            font_em_size,
            short_line_height,
            max_texture_size,
            mip_cell_size,

            canvas_state: CanvasState::new(short_line_height as i32, canvas.size),
            canvases: vec![canvas],
            glyphs: HashMap::new(),
        }
    }

    pub fn font_em_size(&self) -> f32 {
        self.font_em_size
    }

    pub fn canvas_size(&self) -> Vector2I {
        self.canvas_state.canvas_size
    }

    pub fn canvases(&self) -> &[Canvas] {
        &self.canvases
    }

    /// Prepares glyphs and returns list of UV location
    pub fn prepare_glyphs(
        &mut self,
        glyphs: &[(&Arc<Font>, u32)],
    ) -> Result<(Vec<GlyphInfo>, bool), GlyphLoadingError> {
        let hinting = HintingOptions::None;
        let options = RasterizationOptions::GrayscaleAa;

        let mut result = vec![GlyphInfo::default(); glyphs.len()];
        let mut glyphs_to_add = HashMap::new();

        for (i, &(font, glyph_id)) in glyphs.iter().enumerate() {
            let id = GlyphId(Arc::downgrade(font), glyph_id);
            if let Some(&info) = self.glyphs.get(&id) {
                result[i] = info;
            } else {
                glyphs_to_add.entry(id).or_insert_with(Vec::new).push(i);
            }
        }

        if !glyphs_to_add.is_empty() {
            // process glyphs not added to this atlas
            let mut short_rasterize_information = vec![];
            let mut tall_rasterize_information = vec![];

            struct RasterizeInformation {
                font: Arc<Font>,
                glyph_id: u32,
                rasterize_offset: Vector2I,
                rasterize_size: Vector2I,
                advance: Vector2F,
                // canvas id, position, offset for raster
                rasterize_position: Option<(usize, Vector2I)>,
            }

            for &GlyphId(ref font, glyph_id) in glyphs_to_add.keys() {
                let font = Weak::upgrade(font).unwrap(); // held by glyphs span

                let raster_scale = self.font_em_size / font.metrics().units_per_em as f32;
                let typographic_bounds = font
                    .typographic_bounds(glyph_id)
                    .expect("Error getting glyph info");
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
                let advance = font.advance(glyph_id).unwrap() * raster_scale;

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
                                    vec2i(information.rasterize_size.x(), 0);
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
                        canvas_state.short_glyph_cursor.y() + self.short_line_height as i32,
                    );
                    if canvas_state.short_glyph_cursor.y() > self.canvas_state.tall_glyph_line_min_y
                    {
                        // We don't have space for new line in this canvas so we create new canvas.
                        // TODO: We should resize the canvas size if current canvas is the first canvas in the row.
                        self.canvases
                            .push(Canvas::new(self.canvas_state.canvas_size, Format::A8));
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
                            if canvas_state.tall_glyph_cursor.x() + information.rasterize_size.x()
                                < canvas_state.canvas_size.x()
                                && canvas_state.tall_glyph_cursor.y()
                                    - information.rasterize_size.y()
                                    >= canvas_state.short_glyph_cursor.y()
                            {
                                // There's space for this glyph in current glyph line so add to this line
                                information.rasterize_position =
                                    Some((canvas_index, canvas_state.tall_glyph_cursor));
                                canvas_state.tall_glyph_cursor +=
                                    vec2i(information.rasterize_size.x(), 0);
                                canvas_state.tall_glyph_line_min_y =
                                    canvas_state.tall_glyph_line_min_y.min(
                                        canvas_state.tall_glyph_cursor.y()
                                            - information.rasterize_size.y(),
                                    );
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
                            vec2i(0, canvas_state.tall_glyph_line_min_y);
                    } else {
                        // This means we couldn't insert no characters to the last line due to height problem
                        // so we should move to next canvas
                        // TODO: We should resize the canvas size if current canvas is the first canvas in the row.
                        self.canvases
                            .push(Canvas::new(self.canvas_state.canvas_size, Format::A8));
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
                    &mut self.canvases[canvas_id],
                    information.glyph_id,
                    self.font_em_size,
                    Transform2F::from_translation(
                        (rasterize_position + information.rasterize_offset).to_f32(),
                    ),
                    hinting,
                    options,
                )?;

                let id = GlyphId(Arc::downgrade(&information.font), information.glyph_id);
                let glyph_info = GlyphInfo {
                    canvas_id,
                    glyph_id: information.glyph_id,
                    advance: information.advance,
                    rasterize_offset: information.rasterize_offset,
                    glyph_origin: rasterize_position,
                    glyph_size: information.rasterize_size,
                };
                for &index in &glyphs_to_add[&id] {
                    result[index] = glyph_info;
                }
                self.glyphs.insert(id, glyph_info);
            }
        }

        Ok((result, !glyphs_to_add.is_empty()))
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
#[derive(Debug)]
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
        println!("alloc_texture");
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
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAX_LEVEL, 0);
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
            let poly_rect =
                RectI::new(info.rasterize_offset * vec2i(-1, 1), info.glyph_size).to_f32();
            let uv_rect =
                RectI::new(info.glyph_origin, info.glyph_size * vec2i(1, -1)).to_f32() * uv_scale;

            macro_rules! point {
                ($f: ident) => {
                    PointInfo {
                        pos: (transform * poly_rect.$f()).0.0,
                        uv: uv_rect.$f().0.0,
                        tex: info.canvas_id as f32,
                    }
                };
            }

            points.push(point!(upper_right));
            points.push(point!(lower_left));
            points.push(point!(origin));
            points.push(point!(lower_left));
            points.push(point!(upper_right));
            points.push(point!(lower_right));
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
    pub fn draw_text_simple(
        &mut self,
        atlas: &mut FontAtlas,
        font: &Arc<Font>,
        color: ColorF,
        transform: Transform2F,
        text: &str,
    ) {
        let glyphs = text
            .chars()
            .map(|c| font.glyph_for_char(c).unwrap())
            .collect::<Vec<_>>();
        let (glyph_info, update) = atlas
            .prepare_glyphs(&glyphs.iter().map(|&g| (font, g)).collect::<Vec<_>>())
            .unwrap();
        if update {
            self.update_texture(&atlas);
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
    }
}

unsafe fn compile_shader(type_: GLenum, script: &str) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(type_);
        gl::ShaderSource(
            shader,
            1,
            &script.as_ptr().cast::<i8>(),
            &(script.len() as GLint),
        );
        gl::CompileShader(shader);

        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut info = Vec::new();
            let mut len: GLsizei = 512;
            while info.capacity() < (len as usize) {
                info.reserve(len as usize);
                gl::GetShaderInfoLog(shader, info.capacity() as _, &mut len, info.as_mut_ptr());
            }
            info.set_len(len as usize);
            panic!(
                "compile error: (0x{:x}): {}",
                success,
                String::from_utf8_unchecked(std::mem::transmute(info))
            );
        }

        shader
    }
}

unsafe fn link_shader(shaders: &[GLuint]) -> GLuint {
    unsafe {
        let shader_program = gl::CreateProgram();
        for shader in shaders {
            gl::AttachShader(shader_program, *shader);
        }
        gl::LinkProgram(shader_program);

        let mut success = 0;
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);

        if success == 0 {
            let mut info = Vec::new();
            let mut len: GLsizei = 512;
            while info.capacity() < (len as usize) {
                info.reserve(len as usize);
                gl::GetProgramInfoLog(
                    shader_program,
                    info.capacity() as _,
                    &mut len,
                    info.as_mut_ptr(),
                );
            }
            info.set_len(len as usize);
            panic!(
                "link error: (0x{:x}): {}",
                success,
                String::from_utf8_unchecked(std::mem::transmute(info))
            );
        }

        shader_program
    }
}
