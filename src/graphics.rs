use crate::config::{CompletionOverlayConfig, RingOverlayConfig};
use crate::{KeyboardStatus, LeftRight};
use glam::Vec2;
use skia_safe::colors::{BLACK, TRANSPARENT};
use skia_safe::paint::Style;
use skia_safe::textlayout::{
    FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextAlign, TextDecoration,
    TextStyle,
};
use skia_safe::{op, scalar, Canvas, Color4f, Paint, Point, Rect, Surface};
use std::f32::consts::{FRAC_1_SQRT_2, PI};

pub fn draw_background_ring(
    canvas: &mut Canvas,
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
    canvas: &mut Canvas,
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
    return [
        Point::new(0.0, -axis),
        Point::new(diagonal, -diagonal),
        Point::new(axis, 0.0),
        Point::new(diagonal, diagonal),
        Point::new(0.0, axis),
        Point::new(-diagonal, diagonal),
        Point::new(-axis, 0.0),
        Point::new(-diagonal, -diagonal),
    ];
}

fn render_ring_chars<'a>(
    canvas: &mut Canvas,
    fonts: &FontCollection,
    font_families: &[impl AsRef<str>],
    center: Point,
    size: scalar,
    get_char: impl Fn(i8) -> (&'a str, Color4f, scalar),
) {
    let font_size = size * 0.4;
    let offsets = calc_offsets(size);

    for i in 0..8 {
        let pair = get_char(i);

        // first, compute actual font size
        let computed_font_size: scalar = {
            let mut paragraph = ParagraphBuilder::new(
                ParagraphStyle::new().set_text_style(
                    TextStyle::new()
                        .set_font_size(font_size)
                        .set_font_families(font_families),
                ),
                fonts,
            )
            .add_text(&pair.0)
            .build();
            paragraph.layout(10000 as _);
            let width = paragraph.max_intrinsic_width() + 1.0;
            let computed_font_size = font_size * font_size / width;
            computed_font_size.min(font_size)
        };

        let width = (font_size + 10.0) * pair.2;
        let actual_font_size = computed_font_size * pair.2;

        let mut paragraph = ParagraphBuilder::new(
            ParagraphStyle::new()
                .set_text_align(TextAlign::Center)
                .set_text_style(
                    TextStyle::new()
                        .set_color(pair.1.to_color())
                        .set_font_size(actual_font_size)
                        .set_font_families(font_families),
                ),
            fonts,
        )
        .add_text(&pair.0)
        .build();

        paragraph.layout(width);
        let text_center_pos = center + offsets[i as usize];
        let text_pos = text_center_pos - Point::new(width / 2.0, paragraph.height() / 2.0);
        paragraph.paint(canvas, text_pos);
    }
}

pub fn draw_ring<const is_left: bool, const always_show_in_circle: bool>(
    status: &KeyboardStatus,
    config: &RingOverlayConfig,
    fonts: &FontCollection,
    font_families: &[impl AsRef<str>],
    surface: &mut Surface,
) {
    let side = if is_left {
        LeftRight::Left
    } else {
        LeftRight::Right
    };
    surface.canvas().clear(TRANSPARENT);

    let (current, opposite) = status.get_selecting(side);

    let stick_pos = status.stick_pos(side);

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

    let get_color = |idx: i8| -> Color4f {
        if current == -1 {
            config.normal_char_color
        } else if idx == current {
            config.selecting_char_color
        } else {
            config.un_selecting_char_color
        }
    };

    let (line_step, line_len): (usize, usize) = match side {
        LeftRight::Left => (8, 1),
        LeftRight::Right => (1, 8),
    };

    if always_show_in_circle || opposite == -1 {
        let offsets = calc_offsets(radius);
        #[derive(Clone)]
        struct RingChar<'a> {
            show: &'a str,
            color: Color4f,
            size: scalar,
        }
        impl<'a> Default for RingChar<'a> {
            fn default() -> Self {
                Self {
                    show: Default::default(),
                    color: TRANSPARENT,
                    size: Default::default(),
                }
            }
        }
        #[derive(Clone, Default)]
        struct RingInfo<'a> {
            ring_size: scalar,
            chars: [RingChar<'a>; 8],
        }
        let mut prove: [RingInfo; 8] = Default::default();

        // general case. color is not inited because it always depends on current & opposite
        for (pos_us, ring) in prove.iter_mut().enumerate() {
            let col_origin = line_step * pos_us;
            ring.ring_size = 0.2 * radius;
            for (idx_us, char) in ring.chars.iter_mut().enumerate() {
                char.show = {
                    let key = status.method.table[col_origin + line_len * idx_us].0;
                    key.first().map(|x| x.shows).unwrap_or("")
                };
                char.size = 1.0;
            }
        }

        if current == -1 {
            // if current is not selected color
            for ring in prove.iter_mut() {
                for char in ring.chars.iter_mut() {
                    char.color = config.normal_char_color;
                }
            }

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
            for ring in prove.iter_mut() {
                for char in ring.chars.iter_mut() {
                    char.color = config.un_selecting_char_color;
                }
            }

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
                    let key = status.method.table[line_step * current + line_len * opposite].0;
                    key.get(status.button_idx).map(|x| x.shows).unwrap_or("")
                };
                char.color = config.selecting_char_in_ring_color;
                char.size = 1.2;
            }
        }

        for (pos, ring) in prove.iter().enumerate() {
            render_ring_chars(
                surface.canvas(),
                &fonts,
                font_families,
                offsets[pos] + center,
                ring.ring_size,
                |idx| {
                    let char = &ring.chars[idx as usize];
                    (char.show, char.color, char.size)
                },
            )
        }
    } else {
        let line_origin = line_len * opposite as usize;
        render_ring_chars(
            surface.canvas(),
            &fonts,
            font_families,
            center,
            radius,
            |idx| {
                (
                    status.method.table[line_origin + line_step * idx as usize]
                        .0
                        .first()
                        .map(|x| x.shows)
                        .unwrap_or(""),
                    get_color(idx),
                    if idx == current { 1.1 } else { 1.0 },
                )
            },
        )
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
    fonts: &FontCollection,
    font_families: &[impl AsRef<str>],
    surface: &mut Surface,
) {
    surface.canvas().clear(TRANSPARENT);

    const SPACE_RATIO: scalar = 0.1;
    const FONT_SIZE_RATIO: scalar = 0.7;

    fn render(
        canvas: &mut Canvas,
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
                style.set_font_families(font_families);
                style.set_font_size(font_size);
                style
            };
            let not_changing = {
                let mut style: TextStyle = style.clone();
                style.decoration_mut().ty |= TextDecoration::UNDERLINE;
                style
            };

            let changing = {
                let mut style: TextStyle = style.clone();
                style.decoration_mut().ty |= TextDecoration::UNDERLINE;
                style.set_color(config.inputting_char_color.to_color());
                style
            };

            let mut builder = ParagraphBuilder::new(
                &ParagraphStyle::new()
                    .set_text_align(TextAlign::Left)
                    .set_max_lines(1)
                    .set_text_style(&style),
                fonts,
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
            style.set_font_families(font_families);
            style.set_font_size(font_size);
            style
        };

        let paragraphs = recommendations
            .iter()
            .map(|txt| {
                let mut builder = ParagraphBuilder::new(
                    &ParagraphStyle::new()
                        .set_text_align(TextAlign::Left)
                        .set_max_lines(1)
                        .set_text_style(&style),
                    fonts,
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
            - space * 4.0;

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
