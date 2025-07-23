use crate::KeyboardStatus;
use crate::config::{CompletionOverlayConfig, RingOverlayConfig};
use crate::font_rendering::{Font, FontAtlas, FontMetrics, FontRenderer, Layout, TextArranger};
use crate::gl_primitives::{BaseBackgroundRenderer, CircleRenderer, RectangleRenderer};
use crate::input_method::CleKeyButton;
use font_kit::handle::Handle;
use glam::Vec2;
use log::error;
use pathfinder_color::ColorF;
use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::transform2d::{Matrix2x2F, Transform2F};
use pathfinder_geometry::vector::{Vector2F, vec2f};
use std::array::from_fn;
use std::f32::consts::FRAC_1_SQRT_2;
use std::sync::Arc;

/// This context holds multiple renderers, that holds shader, uniform and other information
pub struct GraphicsContext {
    font_atlas: FontAtlas,
    font_renderer: FontRenderer,
    font_layout: TextArranger,

    circle_renderer: CircleRenderer,
    rectangle_renderer: RectangleRenderer,
    base_background_renderer: BaseBackgroundRenderer,

    to_background_channel_sender: std::sync::mpsc::Sender<Vec<(Arc<Font>, u32)>>,
    from_background_channel_receiver: std::sync::mpsc::Receiver<FontAtlas>,
}

impl GraphicsContext {
    pub fn new() -> Self {
        let (to_background_channel_sender, _) = std::sync::mpsc::channel::<Vec<_>>();
        let (_, from_background_channel_receiver) = std::sync::mpsc::channel();

        let mut context = Self {
            font_atlas: FontAtlas::new(200.0, 65536, 16),
            font_renderer: FontRenderer::new(),
            font_layout: TextArranger::new([
                Handle::from_memory(
                    Arc::new(Vec::from(include_bytes!("../fonts/NotoSansJP-Medium.otf"))),
                    0,
                ),
                Handle::from_memory(
                    Arc::new(Vec::from(include_bytes!(
                        "../fonts/NotoEmoji-VariableFont_wght.ttf"
                    ))),
                    0,
                ),
                Handle::from_memory(
                    Arc::new(Vec::from(include_bytes!(
                        "../fonts/NotoSansSymbols2-Regular.ttf"
                    ))),
                    0,
                ),
            ])
            .expect("loading fonts"),

            circle_renderer: CircleRenderer::new(),
            rectangle_renderer: RectangleRenderer::new(),
            base_background_renderer: BaseBackgroundRenderer::new(),

            to_background_channel_sender,
            from_background_channel_receiver,
        };

        // TODO: render glyphs on build to speed-up startup?
        for text in include_str!("glyphs_to_load_initially.txt").chars() {
            context.send_glyphs(
                context
                    .font_layout
                    .layout(text.encode_utf8(&mut [0; 4]), &[])
                    .glyphs()
                    .to_vec(),
            );
        }

        context.font_renderer.update_texture(&context.font_atlas);

        context
    }

    pub fn receive_atlas(&mut self) {
        if let Ok(atlas) = self.from_background_channel_receiver.try_recv() {
            self.font_atlas = atlas;
            while let Ok(atlas) = self.from_background_channel_receiver.try_recv() {
                self.font_atlas = atlas;
            }

            self.font_renderer.update_texture(&self.font_atlas);
        }
    }

    fn send_glyphs(&mut self, mut glyphs: Vec<(Arc<Font>, u32)>) {
        while let Err(e) = self.to_background_channel_sender.send(glyphs) {
            glyphs = e.0;
            let to_background_channel_receiver;
            let from_background_channel_sender;

            (
                self.to_background_channel_sender,
                to_background_channel_receiver,
            ) = std::sync::mpsc::channel::<Vec<_>>();
            (
                from_background_channel_sender,
                self.from_background_channel_receiver,
            ) = std::sync::mpsc::channel();

            std::thread::spawn({
                let mut font_atlas = self.font_atlas.clone();
                move || {
                    log::debug!("graphics context background thread: started");
                    while let Ok(glyphs_to_add) = to_background_channel_receiver.recv() {
                        match font_atlas.rasterize_glyphs(&glyphs_to_add) {
                            Ok(true) => {
                                log::debug!("graphics context background thread: updated glyphs");
                                if from_background_channel_sender
                                    .send(font_atlas.clone())
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            Ok(false) => {}
                            Err(e) => {
                                error!("Error while rasterize font: {e}");
                            }
                        }
                    }
                    log::debug!("graphics context background thread: exiting");
                }
            });
        }
    }

    pub fn render_text(&mut self, color: ColorF, layout: &Layout) {
        let (glyphs, updated) = self.font_atlas.get_glyphs(layout.glyphs()).unwrap();

        self.send_glyphs(updated);

        self.font_renderer.draw_glyphs(
            color,
            glyphs.into_iter().zip(layout.transforms().iter().copied()),
        );
    }
}

#[derive(Clone)]
struct RingChar<'a> {
    show: &'a str,
    color: ColorF,
    size: f32,
}
#[derive(Clone)]
struct RingInfo<'a> {
    ring_size: f32,
    chars: [RingChar<'a>; 8],
}

pub fn draw_cursor_circle(context: &GraphicsContext, stick: Vec2, color: ColorF) {
    let stick = Vector2F::new(stick.x, stick.y);
    context.circle_renderer.draw(
        Transform2F::from_scale_rotation_translation(Vector2F::splat(0.25), 0.0, stick),
        color,
    );
}

fn calc_offsets(size: f32) -> [Vector2F; 8] {
    let axis = 0.75 * size;
    let diagonal = axis * FRAC_1_SQRT_2;
    [
        Vector2F::new(0.0, axis),
        Vector2F::new(diagonal, diagonal),
        Vector2F::new(axis, 0.0),
        Vector2F::new(diagonal, -diagonal),
        Vector2F::new(0.0, -axis),
        Vector2F::new(-diagonal, -diagonal),
        Vector2F::new(-axis, 0.0),
        Vector2F::new(-diagonal, diagonal),
    ]
}

fn render_text_in_box(
    context: &mut GraphicsContext,
    box_size: Vector2F,
    text: &str,
    color: ColorF,
    center: Vector2F,
) {
    let metrics = context.font_layout.metrics();
    let mut layout = context.font_layout.layout(text, &[]);
    // first, compute actual font size
    let computed_font_size: f32 = {
        // assumption: height of characters are 1em.
        let char_height = 1.;
        let y_scale = box_size.y() / char_height;
        let x_scale = box_size.x() / layout.cursor_advance().x();
        y_scale.min(x_scale)
    };
    let text_transform = Transform2F::from_translation(vec2f(
        -layout.cursor_advance().x() * 0.5,
        -metrics.cap_height * 0.5,
    ));
    let text_transform = Transform2F::from_scale(computed_font_size) * text_transform;
    let text_transform = Transform2F::from_translation(center) * text_transform;

    layout.apply_transform(text_transform);

    context.render_text(color, &layout);
}

fn render_ring_chars(context: &mut GraphicsContext, center: Vector2F, ring: &RingInfo) {
    let font_size = ring.ring_size * 0.4;
    let offsets = calc_offsets(ring.ring_size);

    for (i, char) in ring.chars.iter().enumerate() {
        render_text_in_box(
            context,
            Vector2F::splat(font_size * char.size),
            char.show,
            char.color,
            center + offsets[i],
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_ring<'a, const ALWAYS_SHOW_IN_CIRCLE: bool>(
    context: &mut GraphicsContext,
    config: &RingOverlayConfig,
    button_idx: usize,
    current: i8,
    opposite: i8,
    stick_pos: Vec2,
    get_key: impl Fn(/*cur*/ usize, /*oppo*/ usize) -> CleKeyButton<'a>,
) {
    crate::gl_primitives::gl_clear(ColorF::transparent_black());

    let radius = 1.0;

    context.base_background_renderer.draw(
        Transform2F::default(),
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
            render_ring_chars(context, offsets[pos], ring)
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
        render_ring_chars(context, Vector2F::zero(), &ring)
    }

    draw_cursor_circle(context, stick_pos, ColorF::new(0.22, 0.22, 0.22, 1.0));
}

pub fn draw_center(
    status: &KeyboardStatus,
    config: &CompletionOverlayConfig,
    context: &mut GraphicsContext,
) {
    crate::gl_primitives::gl_clear(ColorF::transparent_black());
    const SPACE_RATIO: f32 = 0.1;
    const FONT_SIZE_RATIO: f32 = 0.7;

    let width = 2.0;
    let lane_height = 0.36;
    let space_x = lane_height * SPACE_RATIO * 0.5;
    let font_size = lane_height * FONT_SIZE_RATIO;
    let font_size = vec2f(font_size * 0.5, font_size);

    context.rectangle_renderer.draw(
        RectF::new(vec2f(-1.0, 1.0), vec2f(width, -lane_height)),
        0.,
        config.background_color,
    );

    let max_width = 2.0 - 2.0 * space_x;

    if status.candidates.is_empty() {
        let metrics = context.font_layout.metrics();
        let mut layout = context.font_layout.layout(&status.buffer, &[]);
        let color = config.inputting_char_color;

        let mut cursor = vec2f(-1. + space_x, 1. - lane_height);
        cursor.0[1] += lane_height / 2.0;
        cursor.0[1] -= metrics.cap_height * font_size.y() / 2.0;

        let length = layout.cursor_advance().x() * font_size.x() + font_size.x();
        if length > max_width {
            cursor.0[0] += max_width - length;
        }

        layout.apply_transform(Transform2F {
            matrix: Matrix2x2F::from_scale(font_size),
            vector: cursor,
        });

        context.render_text(color, &layout);
        draw_underline(
            &context.rectangle_renderer,
            font_size,
            &layout,
            metrics,
            color,
        );
    } else {
        let metrics = context.font_layout.metrics();
        let mut advance = 0.0;
        let layouts = status
            .candidates
            .iter()
            .map(|can| {
                let layout = context.font_layout.layout(&can.candidates[can.index], &[]);
                let result = (advance, layout);
                advance += result.1.cursor_advance().x();
                result
            })
            .collect::<Vec<_>>();

        let mut cursor_x;
        if advance * font_size.x() < max_width {
            // for short text, align to first
            cursor_x = -1. + space_x;
        } else {
            let current_candidate = &layouts[status.candidates_idx];
            cursor_x = -(current_candidate.0 + current_candidate.1.cursor_advance().x() * 0.5)
                * font_size.x();
            cursor_x = cursor_x.min(-1. + space_x);
            cursor_x = cursor_x.max(-advance * font_size.x() + 1.0 - space_x);
        }

        let mut cursor_y = 1. - lane_height;
        cursor_y += lane_height / 2.0;
        cursor_y -= metrics.cap_height * font_size.y() / 2.0;

        let mut cursor = vec2f(cursor_x, cursor_y);

        for (i, (_, mut layout)) in layouts.into_iter().enumerate() {
            let color = if i == status.candidates_idx {
                config.inputting_char_color
            } else {
                ColorF::black()
            };

            layout.apply_transform(Transform2F {
                matrix: Matrix2x2F::from_scale(font_size),
                vector: cursor,
            });

            cursor += layout.cursor_advance();

            context.render_text(color, &layout);
            draw_underline(
                &context.rectangle_renderer,
                font_size,
                &layout,
                metrics,
                color,
            );
        }
    }

    if !status.candidates.is_empty() {
        // rendering selections
        let base = lane_height;
        let lane_height = 2.0 * 0.13;
        let font_size = lane_height * FONT_SIZE_RATIO;
        let space = lane_height * SPACE_RATIO;
        let font_size = vec2f(font_size * 0.5, font_size);

        let candidates = status.candidates[status.candidates_idx]
            .candidates
            .as_slice();

        let metrics = context.font_layout.metrics();

        let layouts = candidates
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let mut layout = context.font_layout.layout(text, &[]);

                let mut cursor = vec2f(-1.0, 1. - (base + lane_height * (i as f32)) - lane_height);
                cursor += vec2f(space, 0.);
                cursor.0[1] += lane_height / 2.0;
                cursor.0[1] -= metrics.cap_height * font_size.y() / 2.0;

                layout.apply_transform(Transform2F {
                    matrix: Matrix2x2F::from_scale(font_size),
                    vector: cursor,
                });

                layout
            })
            .collect::<Vec<_>>();

        let width = layouts
            .iter()
            .map(|x| x.cursor_advance().x())
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(0.0)
            + space * 2.0;

        context.rectangle_renderer.draw(
            RectF::new(
                vec2f(-1.0, 1. - base),
                vec2f(width, -lane_height * candidates.len() as f32),
            ),
            0.0,
            config.background_color,
        );

        for layout in layouts.iter() {
            context.render_text(config.inputting_char_color, layout);
        }
    }
}

fn draw_underline(
    rectangle_renderer: &RectangleRenderer,
    font_size: Vector2F,
    layout: &Layout,
    metrics: FontMetrics,
    color: ColorF,
) {
    let underline_space = font_size.x() * 0.05;
    if layout.cursor_advance().x() > underline_space * 2.0 {
        let cursor = layout.transform().vector;
        let underline = RectF::new(
            cursor - (vec2f(0.0, -metrics.underline_position)) * font_size
                + vec2f(underline_space, 0.0),
            (vec2f(0.0, -metrics.underline_thickness)) * font_size
                + layout.cursor_advance()
                + vec2f(-2.0 * underline_space, 0.0),
        );

        rectangle_renderer.draw(underline, 0.0, color);
    }
}
