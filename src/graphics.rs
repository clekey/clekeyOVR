use crate::KeyboardStatus;
use crate::config::{CompletionOverlayConfig, RingOverlayConfig};
use crate::input_method::CleKeyButton;
use glam::Vec2;
use pathfinder_canvas::{
    ArcDirection, CanvasRenderingContext2D, FillRule, Path2D, RectF, TextAlign, TextBaseline,
    Vector2F, vec2f,
};
use pathfinder_color::ColorU;
use std::array::from_fn;
use std::f32::consts::{FRAC_1_SQRT_2, PI};

pub type FontInfo<'a> = [&'a str];

#[derive(Clone)]
struct RingChar<'a> {
    show: &'a str,
    color: ColorU,
    size: f32,
}
#[derive(Clone)]
struct RingInfo<'a> {
    ring_size: f32,
    chars: [RingChar<'a>; 8],
}

pub fn draw_background_ring(
    canvas: &mut CanvasRenderingContext2D,
    center: Vector2F,
    radius: f32,
    center_color: ColorU,
    background_color: ColorU,
    edge_color: ColorU,
) {
    let edge_width = radius * 0.04;
    let background_radius = radius - edge_width / 2.0;

    // background
    let mut path = Path2D::new();
    path.arc(center, radius, 0.0, 360.0, ArcDirection::CW);
    canvas.set_fill_style(background_color);
    canvas.fill_path(path, FillRule::Winding);

    // edge
    let mut path = Path2D::new();
    path.arc(center, background_radius, 0.0, 360.0, ArcDirection::CW);

    macro_rules! draw_lines {
            ($(($a: expr, $b: expr)),* $(,)?) => {
                $(
                    path.move_to(center + vec2f($a, $b));
                    path.line_to(center - vec2f($a, $b));
                )*
            };
        }
    let x = (PI / 8.0).sin() * background_radius;
    let y = (PI / 8.0).cos() * background_radius;
    draw_lines!(
        (x, y),
        (x, -y),
        (-x, y),
        (-x, -y),
        (y, x),
        (-y, x),
        (y, -x),
        (-y, -x),
    );

    canvas.set_line_width(edge_width);
    canvas.set_stroke_style(edge_color);
    canvas.stroke_path(path);

    let mut path = Path2D::new();
    path.arc(center, radius / 2.0, 0.0, 360.0, ArcDirection::CW);
    canvas.set_fill_style(center_color);
    canvas.fill_path(path, FillRule::Winding);
}

pub fn draw_cursor_circle(
    canvas: &mut CanvasRenderingContext2D,
    center: Vector2F,
    radius: f32,
    stick: Vec2,
    color: ColorU,
) {
    let stick = Vector2F::new(stick.x, -stick.y);
    let mut path = Path2D::new();
    path.arc(
        center + stick * (radius / 4.0),
        radius / 4.0,
        0.0,
        360.0,
        ArcDirection::CW,
    );
    canvas.set_fill_style(color);
    canvas.fill_path(path, FillRule::Winding);
}

fn calc_offsets(size: f32) -> [Vector2F; 8] {
    let axis = 0.75 * size;
    let diagonal = axis * FRAC_1_SQRT_2;
    [
        Vector2F::new(0.0, -axis),
        Vector2F::new(diagonal, -diagonal),
        Vector2F::new(axis, 0.0),
        Vector2F::new(diagonal, diagonal),
        Vector2F::new(0.0, axis),
        Vector2F::new(-diagonal, diagonal),
        Vector2F::new(-axis, 0.0),
        Vector2F::new(-diagonal, -diagonal),
    ]
}

fn render_text_in_box(
    canvas: &mut CanvasRenderingContext2D,
    fonts: &FontInfo,
    box_size: f32,
    text: &str,
    color: ColorU,
    center: Vector2F,
) {
    canvas.set_font(fonts).expect("font not found");
    canvas.set_font_size(box_size);
    canvas.set_text_align(TextAlign::Center);

    let metrics = canvas.measure_text(text);
    let text_bounds = vec2f(
        metrics.width(),
        metrics.font_bounding_box_ascent() - metrics.font_bounding_box_descent(),
    );
    let scale = (box_size / text_bounds.x().max(text_bounds.y())).min(1.0);
    let position = center
        - vec2f(
            metrics.actual_bounding_box_left() + metrics.actual_bounding_box_right(),
            -(metrics.font_bounding_box_ascent() + metrics.font_bounding_box_descent()),
        ) * 0.5
            * scale;
    canvas.set_font_size(box_size * scale);

    canvas.set_fill_style(color);
    canvas.fill_text(text, position);
}

fn render_ring_chars<'a>(
    canvas: &mut CanvasRenderingContext2D,
    fonts: &FontInfo,
    center: Vector2F,
    ring: &RingInfo,
) {
    let font_size = ring.ring_size * 0.4;
    let offsets = calc_offsets(ring.ring_size);

    for (i, char) in ring.chars.iter().enumerate() {
        render_text_in_box(
            canvas,
            fonts,
            font_size * char.size,
            char.show,
            char.color,
            center + offsets[i],
        );
    }
}

pub(crate) fn draw_ring<'a, const ALWAYS_SHOW_IN_CIRCLE: bool>(
    canvas: &mut CanvasRenderingContext2D,
    config: &RingOverlayConfig,
    fonts: &FontInfo,
    button_idx: usize,
    current: i8,
    opposite: i8,
    stick_pos: Vec2,
    get_key: impl Fn(/*cur*/ usize, /*oppo*/ usize) -> CleKeyButton<'a>,
) {
    let center = canvas.canvas().size().to_f32() * 0.5;
    let radius = center.x();

    draw_background_ring(
        canvas,
        center,
        radius,
        config.center_color,
        config.background_color,
        config.edge_color,
    );

    if ALWAYS_SHOW_IN_CIRCLE || opposite == -1 {
        let default_color = if current == -1 {
            config.normal_char_color
        } else {
            config.un_selecting_char_color
        };

        // initialize with general case.
        let mut prove: [RingInfo; 8] = from_fn(|pos| RingInfo {
            ring_size: 0.2 * radius,
            chars: from_fn(|idx| RingChar {
                show: {
                    let key = get_key(pos, idx);
                    key.0.first().map(|x| x.shows).unwrap_or("")
                },
                color: default_color,
                size: 1.0,
            }),
        });

        if current == -1 {
            //if current == -1 and opposite is selected
            //  prove[*].chars[opposite].color = config.selecting_char_in_ring_color;
            if opposite != -1 {
                let opposite = opposite as usize;
                for ring in prove.iter_mut() {
                    ring.chars[opposite].color = config.selecting_char_in_ring_color;
                }
            }
        } else {
            let current = current as usize;

            // for selecting ring, size is 0.22 && set color to selecting_char_color
            let ring = &mut prove[current];
            ring.ring_size = 0.22 * radius;
            for char in ring.chars.iter_mut() {
                char.color = config.selecting_char_color;
            }

            // for selecting char, set color to selecting_char_in_ring_color
            if opposite != -1 {
                let opposite = opposite as usize;
                let char = &mut ring.chars[opposite];
                char.show = {
                    let key = get_key(current, opposite);
                    key.0.get(button_idx).map(|x| x.shows).unwrap_or("")
                };
                char.color = config.selecting_char_in_ring_color;
                char.size = 1.2;
            }
        }

        let offsets = calc_offsets(radius);
        for (pos, ring) in prove.iter().enumerate() {
            render_ring_chars(canvas, fonts, offsets[pos] + center, ring)
        }
    } else {
        let default_color = if current == -1 {
            config.normal_char_color
        } else {
            config.un_selecting_char_color
        };

        let mut ring = RingInfo {
            ring_size: radius,
            chars: from_fn(|idx| RingChar {
                show: {
                    let key = get_key(idx, opposite as usize);
                    key.0.first().map(|x| x.shows).unwrap_or("")
                },
                color: default_color,
                size: 1.0,
            }),
        };

        if current != -1 {
            ring.chars[current as usize].color = config.selecting_char_color;
            ring.chars[current as usize].size = 1.1;
        }
        render_ring_chars(canvas, fonts, center, &ring)
    }

    draw_cursor_circle(
        canvas,
        center,
        radius,
        stick_pos,
        ColorU::new(56, 56, 56, 255),
    );
}

pub fn draw_center(
    status: &KeyboardStatus,
    config: &CompletionOverlayConfig,
    fonts: &FontInfo,
    canvas: &mut CanvasRenderingContext2D,
) {
    const SPACE_RATIO: f32 = 0.1;
    const FONT_SIZE_RATIO: f32 = 0.7;

    let width = canvas.canvas().size().x() as f32;
    let lane_height = canvas.canvas().size().y() as f32 * 0.18;
    let space = lane_height * SPACE_RATIO;
    let font_size = lane_height * FONT_SIZE_RATIO;

    // configure font settings
    canvas.set_font(fonts).expect("Failed to set font");
    canvas.set_font_size(font_size);
    canvas.set_text_baseline(TextBaseline::Alphabetic);
    canvas.set_line_width(2.0);

    struct TextRenderer<'a> {
        canvas: &'a mut CanvasRenderingContext2D,
        cursor: Vector2F,
    }
    impl<'a> TextRenderer<'a> {
        fn draw_underlined(&mut self, text: &str, color: ColorU) {
            self.draw_text_inner(text, color, true);
        }

        fn draw_text(&mut self, text: &str, color: ColorU) {
            self.draw_text_inner(text, color, false);
        }

        fn draw_text_inner(&mut self, text: &str, color: ColorU, underline: bool) {
            self.canvas.set_fill_style(color);
            self.canvas.set_stroke_style(color);

            let metrics = self.canvas.measure_text(text);

            if underline {
                static UNDERLINE_SPACE: f32 = 1.0;
                if metrics.width() > UNDERLINE_SPACE * 2.0 {
                    let start = self.cursor + vec2f(UNDERLINE_SPACE, 0.0);
                    let end = self.cursor + vec2f(metrics.width() - UNDERLINE_SPACE, 0.0);
                    let mut path = Path2D::new();
                    path.move_to(start);
                    path.line_to(end);
                    self.canvas.stroke_path(path);
                }
            }

            self.canvas.fill_text(text, self.cursor);

            self.cursor += vec2f(metrics.width(), 0.0);
        }
    }

    let mut text_renderer = TextRenderer {
        canvas,
        cursor: vec2f(space, lane_height - space),
    };

    text_renderer.canvas.set_fill_style(config.background_color);
    text_renderer
        .canvas
        .fill_rect(RectF::new(vec2f(0.0, 0.0), vec2f(width, lane_height)));

    // TODO: scroll horizontally to show the end of input or currently changing text.
    if status.candidates.is_empty() {
        text_renderer.draw_underlined(&status.buffer, config.inputting_char_color);
    } else {
        for (i, can) in status.candidates.iter().enumerate() {
            if i == status.candidates_idx {
                text_renderer
                    .draw_underlined(&can.candidates[can.index], config.inputting_char_color);
            } else {
                text_renderer.draw_underlined(&can.candidates[can.index], ColorU::black());
            }
        }
    }

    let base = lane_height;
    let lane_height = text_renderer.canvas.canvas().size().y() as f32 * 0.13;
    let font_size = lane_height * FONT_SIZE_RATIO;
    let space = lane_height * SPACE_RATIO;
    text_renderer.canvas.set_font_size(font_size);
    if !status.candidates.is_empty() {
        let candidates = status.candidates[status.candidates_idx]
            .candidates
            .as_slice();

        for (i, text) in candidates.iter().enumerate() {
            let metrics = text_renderer.canvas.measure_text(text);
            let width = metrics.width() + space * 2.0;
            let rect = RectF::new(
                vec2f(0.0, base + lane_height * (i as f32)),
                vec2f(width, lane_height),
            );

            text_renderer.canvas.set_fill_style(config.background_color);
            text_renderer
                .canvas
                .fill_rect(RectF::new(vec2f(0.0, 0.0), vec2f(width, lane_height)));

            text_renderer.cursor = rect.lower_left() + vec2f(space, -space);
            text_renderer.draw_text(text, config.inputting_char_color);
        }
    }
}
