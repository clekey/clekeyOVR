use crate::config::{CompletionOverlayConfig, RingOverlayConfig};
use crate::utils::ToTuple;
use crate::{KeyboardStatus, LeftRight};
use skia_safe::colors::TRANSPARENT;
use skia_safe::paint::Style;
use skia_safe::textlayout::{
    FontCollection, ParagraphBuilder, ParagraphStyle, TextAlign, TextStyle,
};
use skia_safe::{scalar, Canvas, Color4f, Paint, Point, Surface, Color};
use std::f32::consts::{FRAC_1_SQRT_2, PI};
use glam::Vec2;

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
        center + stick * (radius / 3.0),
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
    get_char: impl Fn(i8) -> (&'a str, Color4f),
) {
    let font_size = size * 0.4;
    let offsets = calc_offsets(size);

    for i in 0..8 {
        let pair = get_char(i);

        // first, compute actual font size
        let actual_font_size: scalar;
        {
            let mut paragraph = ParagraphBuilder::new(
                ParagraphStyle::new().set_text_style(TextStyle::new().set_font_size(font_size).set_font_families(font_families)),
                fonts,
            )
            .add_text(&pair.0)
            .build();
            paragraph.layout(10000 as _);
            let width = paragraph.max_intrinsic_width() + 1.0;
            let computed_font_size = font_size * font_size / width;
            actual_font_size = computed_font_size.min(font_size);
        }

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
        paragraph.layout(font_size + 10.0);
        let text_center_pos = center + offsets[i as usize];
        let text_pos =
            text_center_pos - Point::new((font_size + 10.0) / 2.0, paragraph.height() / 2.0);
        paragraph.paint(canvas, text_pos);
    }
}

pub fn draw_ring(
    status: &KeyboardStatus,
    side: LeftRight,
    always_show_in_circle: bool,
    config: &RingOverlayConfig,
    fonts: &FontCollection,
    font_families: &[impl AsRef<str>],
    surface: &mut Surface,
) {
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
        for pos in 0..(8 as i8) {
            let col_origin = line_step * pos as usize;
            let ring_color = get_color(pos);
            let ring_size = if pos == current { 0.22 } else { 0.2 } * radius;
            render_ring_chars(
                surface.canvas(),
                &fonts,
                font_families,
                offsets[pos as usize] + center,
                ring_size,
                |idx| {
                    (
                        status.method.get_table()[col_origin + line_len * idx as usize],
                        ring_color,
                    )
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
                    status.method.get_table()[line_origin + line_step * idx as usize],
                    get_color(idx),
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
    surface.canvas().clear(config.background_color);

    let space = surface.height() as scalar * 0.15;

    let mut paragraph = ParagraphBuilder::new(&ParagraphStyle::new()
        .set_text_align(TextAlign::Left)
        .set_text_style(TextStyle::new()
            .set_color(config.inputting_char_color.to_color())
            .set_font_families(font_families)
            .set_font_size(surface.height() as scalar * 0.5)),fonts)
        .add_text(status.method.buffer())
        .build();
    paragraph.layout(surface.width() as scalar - space - space);
    paragraph.paint(surface.canvas(), Point::new(space, space));
}
