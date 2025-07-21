use crate::KeyboardStatus;
use crate::config::{CompletionOverlayConfig, RingOverlayConfig};
use crate::input_method::CleKeyButton;
use glam::Vec2;
use skia_safe::colors::{BLACK, TRANSPARENT};
use skia_safe::paint::Style;
use skia_safe::textlayout::{
    FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextAlign, TextDecoration,
    TextStyle,
};
use skia_safe::{Canvas, Color4f, Paint, Point, Rect, Surface, scalar};
use std::array::from_fn;
use std::f32::consts::{FRAC_1_SQRT_2, PI};

pub struct FontInfo<'a> {
    pub(crate) collection: FontCollection,
    pub(crate) families: &'a [String],
}

#[derive(Clone)]
struct RingChar<'a> {
    show: &'a str,
    color: Color4f,
    size: scalar,
}
#[derive(Clone)]
struct RingInfo<'a> {
    ring_size: scalar,
    chars: [RingChar<'a>; 8],
}

pub fn draw_background_ring(
    canvas: &Canvas,
    center: Point,
    radius: f32,
    center_color: Color4f,
    background_color: Color4f,
    edge_color: Color4f,
) {
    let edge_width = radius * 0.04;
    let background_radius = radius - edge_width / 2.0;

    // background
    canvas.draw_circle(
        center,
        background_radius,
        Paint::new(background_color, None).set_style(Style::Fill),
    );

    // edge
    let mut edge = Paint::new(edge_color, None);
    edge.set_anti_alias(true)
        .set_style(Style::Stroke)
        .set_stroke_width(edge_width);
    canvas.draw_circle(center, background_radius, &edge);

    macro_rules! draw_lines {
            ($(($a: expr, $b: expr)),* $(,)?) => {
                canvas
                    $(.draw_line(center + Point::new(-$a, -$b), center + Point::new($a, $b), &edge))*
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

    canvas.draw_circle(
        center,
        radius / 2.0,
        Paint::new(center_color, None)
            .set_anti_alias(true)
            .set_style(Style::Fill),
    );
}

pub fn draw_cursor_circle(
    canvas: &Canvas,
    center: Point,
    radius: f32,
    stick: Vec2,
    color: Color4f,
) {
    let stick = Point::new(stick.x, -stick.y);
    canvas.draw_circle(
        center + stick * (radius / 4.0),
        radius / 4.0,
        Paint::new(color, None)
            .set_anti_alias(true)
            .set_style(Style::Fill),
    );
}

fn calc_offsets(size: scalar) -> [Point; 8] {
    let axis = 0.75 * size;
    let diagonal = axis * FRAC_1_SQRT_2;
    [
        Point::new(0.0, -axis),
        Point::new(diagonal, -diagonal),
        Point::new(axis, 0.0),
        Point::new(diagonal, diagonal),
        Point::new(0.0, axis),
        Point::new(-diagonal, diagonal),
        Point::new(-axis, 0.0),
        Point::new(-diagonal, -diagonal),
    ]
}

fn render_text_in_box(
    canvas: &Canvas,
    fonts: &FontInfo,
    box_size: scalar,
    text: &str,
    color: Color4f,
    center: Point,
) {
    // first, compute actual font size
    let computed_font_size: scalar = {
        let mut paragraph = ParagraphBuilder::new(
            ParagraphStyle::new().set_text_style(
                TextStyle::new()
                    .set_font_size(box_size)
                    .set_font_families(fonts.families),
            ),
            &fonts.collection,
        )
        .add_text(text)
        .build();
        paragraph.layout(10000 as _);
        let width = paragraph.max_intrinsic_width() + 1.0;
        let computed_font_size = box_size * box_size / width;
        computed_font_size.min(box_size)
    };

    let width = box_size + 10.0;
    let actual_font_size = computed_font_size;

    let mut paragraph = ParagraphBuilder::new(
        ParagraphStyle::new()
            .set_text_align(TextAlign::Center)
            .set_text_style(
                TextStyle::new()
                    .set_color(color.to_color())
                    .set_font_size(actual_font_size)
                    .set_font_families(fonts.families),
            ),
        &fonts.collection,
    )
    .add_text(text)
    .build();

    paragraph.layout(width);
    let text_pos = center - Point::new(width / 2.0, paragraph.height() / 2.0);
    paragraph.paint(canvas, text_pos);
}

fn render_ring_chars(canvas: &Canvas, fonts: &FontInfo, center: Point, ring: &RingInfo) {
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_ring<'a, const ALWAYS_SHOW_IN_CIRCLE: bool>(
    surface: &mut Surface,
    config: &RingOverlayConfig,
    fonts: &FontInfo,
    button_idx: usize,
    current: i8,
    opposite: i8,
    stick_pos: Vec2,
    get_key: impl Fn(/*cur*/ usize, /*oppo*/ usize) -> CleKeyButton<'a>,
) {
    surface.canvas().clear(TRANSPARENT);

    let center = Point::new(surface.width() as scalar, surface.height() as scalar) * 0.5;
    let radius = center.x;

    draw_background_ring(
        surface.canvas(),
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
            render_ring_chars(surface.canvas(), fonts, offsets[pos] + center, ring)
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
        render_ring_chars(surface.canvas(), fonts, center, &ring)
    }

    draw_cursor_circle(
        surface.canvas(),
        center,
        radius,
        stick_pos,
        Color4f::new(0.22, 0.22, 0.22, 1.0),
    );
}

pub fn draw_center(
    status: &KeyboardStatus,
    config: &CompletionOverlayConfig,
    fonts: &FontInfo,
    surface: &mut Surface,
) {
    surface.canvas().clear(TRANSPARENT);

    const SPACE_RATIO: scalar = 0.1;
    const FONT_SIZE_RATIO: scalar = 0.7;

    fn render(
        canvas: &Canvas,
        rect: Rect,
        background_color: Color4f,
        paragraph: impl FnOnce(/*font_size: */ f32) -> Paragraph,
    ) {
        let space = rect.height() * SPACE_RATIO;
        let font_size = rect.height() * FONT_SIZE_RATIO;

        canvas.draw_rect(rect, &Paint::new(background_color, None));

        let mut paragraph = paragraph(font_size);
        paragraph.layout(rect.width() - space - space);

        paragraph.paint(
            canvas,
            Point::new(rect.left() + space, rect.top() + space * 2.0),
        );
    }

    let width = surface.width() as scalar;
    let lane_height = surface.height() as scalar * 0.18;

    render(
        surface.canvas(),
        Rect::from_xywh(0.0, 0.0, width, lane_height),
        config.background_color,
        |font_size| {
            let style = {
                let mut style = TextStyle::new();
                style.set_color(BLACK.to_color());
                style.set_height_override(true);
                style.set_height(1.0);
                style.set_font_families(fonts.families);
                style.set_font_size(font_size);
                style
            };
            let not_changing = {
                let mut style: TextStyle = style.clone();
                style.set_decoration_type(TextDecoration::UNDERLINE);
                style
            };

            let changing = {
                let mut style: TextStyle = style.clone();
                style.set_decoration_type(TextDecoration::UNDERLINE);
                style.set_color(config.inputting_char_color.to_color());
                style
            };

            let mut builder = ParagraphBuilder::new(
                ParagraphStyle::new()
                    .set_text_align(TextAlign::Left)
                    .set_max_lines(1)
                    .set_text_style(&style),
                &fonts.collection,
            );

            if status.candidates.is_empty() {
                builder.push_style(&changing);
                builder.add_text(&status.buffer);
                builder.pop();
            } else {
                for (i, can) in status.candidates.iter().enumerate() {
                    if i == status.candidates_idx {
                        builder.push_style(&changing);
                    } else {
                        builder.push_style(&not_changing);
                    }
                    builder.add_text(&can.candidates[can.index]);
                    builder.pop();

                    builder.add_text(" ");
                }
            }

            builder.build()
        },
    );

    let base = lane_height;
    let lane_height = surface.height() as scalar * 0.13;
    let font_size = lane_height * FONT_SIZE_RATIO;
    let space = lane_height * SPACE_RATIO;
    if !status.candidates.is_empty() {
        let recommendations = status.candidates[status.candidates_idx]
            .candidates
            .as_slice();

        let style = {
            let mut style = TextStyle::new();
            style.set_color(config.inputting_char_color.to_color());
            style.set_height_override(true);
            style.set_height(1.0);
            style.set_font_families(fonts.families);
            style.set_font_size(font_size);
            style
        };

        let paragraphs = recommendations
            .iter()
            .map(|txt| {
                let mut builder = ParagraphBuilder::new(
                    ParagraphStyle::new()
                        .set_text_align(TextAlign::Left)
                        .set_max_lines(1)
                        .set_text_style(&style),
                    &fonts.collection,
                );

                builder.add_text(txt);
                let mut p = builder.build();
                p.layout(width);
                p
            })
            .collect::<Vec<_>>();

        let width = paragraphs
            .iter()
            .map(|x| x.max_intrinsic_width())
            .fold(f32::NAN, f32::max)
            + space * 2.0;

        for (i, p) in paragraphs.into_iter().enumerate() {
            render(
                surface.canvas(),
                Rect::from_xywh(0.0, base + lane_height * (i as scalar), width, lane_height),
                config.background_color,
                |_font_size| p,
            );
        }
    }
}
