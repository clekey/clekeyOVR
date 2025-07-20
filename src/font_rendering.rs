//! This module handles font texture atlasing, and texture layout

use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::error::GlyphLoadingError;
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{Vector2I, vec2i};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::hash::Hash;
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
    canvas_id: usize,
    rasterize_offset: Vector2I,
    glyph_origin: Vector2I,
    glyph_size: Vector2I,
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
