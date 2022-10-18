use std::f32::consts::PI;
use skia_safe::{Canvas, Color4f, Paint, Point};
use skia_safe::paint::Style;

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
        Paint::new(background_color, None)
            .set_style(Style::Fill)
    );

    // edge
    let mut edge = Paint::new(edge_color, None);
    edge.set_anti_alias(true)
        .set_style(Style::Stroke)
        .set_stroke_width(edge_width);
    canvas.draw_circle(center, background_radius, &edge);

    macro_rules! draw_lines {
            ($canvas: expr, $(($a: expr, $b: expr)),* $(,)?) => {
                $canvas
                    $(.draw_line(Point::new(-$a, -$b), Point::new($a, $b), &edge))*
            };
        }
    let x = (PI / 8.0).sin();
    let y = (PI / 8.0).cos();
    draw_lines!(
        canvas,
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

pub fn cursor_circle_renderer(
    canvas: &mut Canvas,
    center: Point,
    radius: f32,
    stick: Point,
    color: Color4f,
) {
    canvas.draw_circle(
        center + stick * (radius / 3.0),
        radius / 4.0,
        Paint::new(color, None)
            .set_anti_alias(true)
            .set_style(Style::Fill)
    );
}
