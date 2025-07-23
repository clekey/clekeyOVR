/*
use crate::font_rendering::{Font, FontAtlas, FontRenderer, TextArranger};
use crate::gl_primitives::{BaseBackgroundRenderer, CircleRenderer, RectangleRenderer, gl_clear};
use font_kit::handle::Handle;
use gl::types::{GLsizei, GLuint};
use glfw::{Context, OpenGlProfileHint, WindowHint};
use pathfinder_color::ColorF;
use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::transform2d::{Matrix2x2F, Transform2F};
use pathfinder_geometry::vector::{Vector2F, Vector2I, vec2f};
use std::ptr::null;
use std::sync::Arc;
use std::time::Instant;

#[path = "../font_rendering.rs"]
mod font_rendering;

#[path = "../gl_primitives.rs"]
mod gl_primitives;

const WINDOW_HEIGHT: i32 = 1024;
const WINDOW_WIDTH: i32 = 1024;

fn main() {
    let mut atlas = FontAtlas::new(50.0, 65536, 16);

    let font = Arc::new(
        Font::new(
            Arc::new(Vec::from(include_bytes!(
                "../../resources/fonts/NotoSansJP-Medium.otf"
            ))),
            0,
        )
        .unwrap(),
    );

    let arranger = TextArranger::new([
        Handle::from_memory(
            Arc::new(Vec::from(include_bytes!(
                "../../resources/fonts/NotoSansJP-Medium.otf"
            ))),
            0,
        ),
        Handle::from_memory(
            Arc::new(Vec::from(include_bytes!(
                "../../resources/fonts/NotoEmoji-VariableFont_wght.ttf"
            ))),
            0,
        ),
        Handle::from_memory(
            Arc::new(Vec::from(include_bytes!(
                "../../resources/fonts/NotoSansSymbols2-Regular.ttf"
            ))),
            0,
        ),
    ])
    .expect("loading fonts");

    let (glyphs, changed) = atlas
        .prepare_glyphs(&[
            (font.clone(), font.glyph_for_char('あ').unwrap()),
            (font.clone(), font.glyph_for_char('い').unwrap()),
            (font.clone(), font.glyph_for_char('う').unwrap()),
        ])
        .unwrap();

    assert!(changed);
    println!("{glyphs:#?}");

    let (_, changed) = atlas
        .prepare_glyphs(&[
            (font.clone(), font.glyph_for_char('あ').unwrap()),
            (font.clone(), font.glyph_for_char('い').unwrap()),
            (font.clone(), font.glyph_for_char('う').unwrap()),
        ])
        .unwrap();
    assert!(!changed);

    let (_, changed) = atlas
        .prepare_glyphs(&[
            (font.clone(), font.glyph_for_char('え').unwrap()),
            (font.clone(), font.glyph_for_char('お').unwrap()),
            (font.clone(), font.glyph_for_char('か').unwrap()),
            (font.clone(), font.glyph_for_char('が').unwrap()),
        ])
        .unwrap();
    assert!(changed);

    // glfw initialization
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(WindowHint::DoubleBuffer(true));
    glfw.window_hint(WindowHint::ContextVersionMajor(4));
    glfw.window_hint(WindowHint::ContextVersionMinor(1));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::Resizable(false));
    glfw.window_hint(WindowHint::CocoaRetinaFramebuffer(false));
    glfw.window_hint(WindowHint::Visible(false));
    glfw.window_hint(WindowHint::ContextNoError(false));

    let (mut window, _events) = glfw
        .create_window(
            WINDOW_WIDTH as _,
            WINDOW_HEIGHT as _,
            "clekeyOVR",
            glfw::WindowMode::Windowed,
        )
        .expect("window creation");
    #[cfg(feature = "debug_control")]
    {
        window.set_key_polling(true);
    }

    window.make_current();

    // gl crate initialization
    gl::load_with(|s| {
        glfw.get_proc_address_raw(s)
            .map(|f| f as _)
            .unwrap_or_default()
    });

    {
        window.make_current();

        let regular_use_ideographs = include_str!("regular_use_utf8.txt");
        let hiragana = [
            (0x3041..=0x306b)
                .map(|x| char::from_u32(x as u32).unwrap())
                .collect::<String>(),
            (0x306c..=0x3094)
                .map(|x| char::from_u32(x as u32).unwrap())
                .collect::<String>(),
        ];
        let test_texts = [
            "な\u{3099} test: na + dakuten",
            "か\u{3099} test: ka + dakuten",
            "が test: ga",
            "￣ overline",
            "_ underscore",
            "あいabc⌫␣\u{1F310}가", // emojis
        ];
        let ideographs_count = regular_use_ideographs.len() / 3;
        let per_line = 50;
        let lines = ideographs_count / per_line;
        let ideographs =
            (0..lines).map(|i| &regular_use_ideographs[i * 3 * per_line..][..3 * per_line]);
        //*
        let chars = (hiragana.iter().map(|x| x.as_str()))
            .chain(test_texts)
            .chain(ideographs)
            .collect::<Vec<_>>();
        // */
        //let chars = test_texts.into_iter().collect::<Vec<_>>();

        // note: opengl coordinate starts at left bottom corner as 0, 0 and increase 1, 1 for right top

        let render_target0 = RenderTargetTexture::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let render_target1 = RenderTargetTexture::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let render_target2 = RenderTargetTexture::new(WINDOW_WIDTH, WINDOW_HEIGHT);

        let mut font_renderer = FontRenderer::new();
        font_renderer.update_texture(&atlas);

        let circle_renderer = CircleRenderer::new();
        let bg_renderer = BaseBackgroundRenderer::new();
        let rectangle_renderer = RectangleRenderer::new();

        //let pos_scale = Vector2F::splat(25.6) / Vector2I::new(WINDOW_WIDTH, WINDOW_HEIGHT).to_f32();
        let pos_scale = Vector2F::splat(40.) / Vector2I::new(WINDOW_WIDTH, WINDOW_HEIGHT).to_f32();
        //let pos_scale = Vector2F::splat(100.0) / Vector2I::new(WINDOW_WIDTH, WINDOW_HEIGHT).to_f32();
        let angle = -0.0f32.to_radians();
        //let angle = 10.0f32.to_radians();
        let matrix = Matrix2x2F::from_scale(pos_scale) * Matrix2x2F::from_rotation(angle);

        // rendering
        let render0_start = Instant::now();
        render_target0.prepare_rendering();
        gl_clear(ColorF::black());

        let mut cursor = vec2f(-1.0, 1.0) - matrix * vec2f(0.0, 1.0);
        for text in chars.as_slice() {
            let mut layout = arranger.layout(text, &[]);

            layout.apply_transform(Transform2F {
                matrix,
                vector: cursor,
            });

            let (glyphs, updated) = atlas.prepare_glyphs(layout.glyphs()).unwrap();

            if updated {
                font_renderer.update_texture(&atlas);
            }

            font_renderer.draw_glyphs(
                ColorF::white(),
                glyphs.into_iter().zip(layout.transforms().iter().copied()),
            );

            cursor -= matrix * vec2f(0.0, 1.0);
        }
        let render0_end = Instant::now();
        render_target0.export_png("canvas0.png");

        println!("rendering 0 took {:?}", render0_end - render0_start);

        let render1_start = Instant::now();
        render_target1.prepare_rendering();
        gl_clear(ColorF::black());

        circle_renderer.draw(
            Transform2F::from_scale_rotation_translation(vec2f(1.0, 0.5), 0.0, vec2f(0.0, 1.0)),
            ColorF::new(0., 0.5, 0.5, 1.0),
        );

        let mut cursor = vec2f(-1.0, 1.0) - matrix * vec2f(0.0, 1.0);
        let metrics = arranger.metrics();
        for text in chars.as_slice() {
            let mut layout = arranger.layout(text, &[]);

            layout.apply_transform(Transform2F {
                matrix,
                vector: cursor,
            });

            let (glyphs, updated) = atlas.prepare_glyphs(layout.glyphs()).unwrap();

            if updated {
                font_renderer.update_texture(&atlas);
            }

            font_renderer.draw_glyphs(
                ColorF::new(1.0, 0.0, 0.0, 1.0),
                glyphs.into_iter().zip(layout.transforms().iter().copied()),
            );

            println!("metrics: {:?}", metrics);
            let underline = RectF::new(
                cursor - matrix * (vec2f(0.0, -metrics.underline_position)),
                matrix * (vec2f(0.0, -metrics.underline_thickness)) + layout.cursor_advance(),
            );
            println!(
                "underline: {:?}",
                underline
                    * pathfinder_geometry::vector::vec2i(WINDOW_WIDTH, WINDOW_HEIGHT).to_f32()
            );

            rectangle_renderer.draw(underline, 0.0, ColorF::new(1.0, 0.0, 0.0, 1.0));

            cursor -= matrix * vec2f(0.0, 1.0 + metrics.line_gap);
        }
        let render1_end = Instant::now();
        println!("rendering 1 took {:?}", render1_end - render1_start);
        render_target1.export_png("canvas1.png");

        let render2_start = Instant::now();
        render_target2.prepare_rendering();
        gl_clear(ColorF::transparent_black());

        rectangle_renderer.draw(
            RectF::new(vec2f(0.0, 0.5), vec2f(1.0, 0.5)),
            10f32,
            ColorF::black(),
        );

        bg_renderer.draw(
            Transform2F::default(),
            ColorF::new(0.83, 0.83, 0.83, 1.0),
            ColorF::new(0.686, 0.686, 0.686, 1.0),
            ColorF::new(1., 1.0, 1.0, 1.0),
        );

        let mut cursor = vec2f(-1.0, 1.0) - matrix * vec2f(0.0, 1.0);
        let mut info_transforms = Vec::with_capacity(chars.iter().map(|x| x.chars().count()).sum());
        for text in chars.as_slice() {
            let mut layout = arranger.layout(text, &[]);
            layout.apply_transform(Transform2F {
                matrix,
                vector: cursor,
            });

            let (glyphs, update) = atlas.prepare_glyphs(layout.glyphs()).unwrap();
            assert!(!update);

            info_transforms.extend(glyphs.into_iter().zip(layout.transforms().iter().copied()));
            cursor -= matrix * vec2f(0.0, 1.0);
        }
        font_renderer.draw_glyphs(ColorF::new(0.0, 1.0, 0.0, 1.0), info_transforms);
        let render2_end = Instant::now();
        println!("rendering 2 took {:?}", render2_end - render2_start);
        render_target2.export_png("canvas2.png");
    }

    for (i, canvas) in atlas.canvases().iter().enumerate() {
        let file = std::fs::File::create(format!("atlas.{}.png", i + 1)).unwrap();
        let mut png = png::Encoder::new(file, canvas.size.x() as u32, canvas.size.y() as u32);
        png.set_color(png::ColorType::Grayscale);
        png.set_depth(png::BitDepth::Eight);
        let mut writer = png.write_header().unwrap();
        writer.write_image_data(&canvas.pixels).unwrap();
        writer.finish().unwrap();
    }
}

struct RenderTargetTexture {
    texture: GLuint,
    framebuffer: GLuint,
    width: GLsizei,
    height: GLsizei,
}

impl RenderTargetTexture {
    pub fn new(width: GLsizei, height: GLsizei) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        unsafe {
            // generate framebuffer with texture
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as _,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);

            let mut framebuffer = 0;

            gl::GenFramebuffers(1, &mut framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, texture, 0);
            gl::DrawBuffers(1, [gl::COLOR_ATTACHMENT0].as_ptr());

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!(
                    "Framebuffer rendering failed: {:x}",
                    gl::CheckFramebufferStatus(gl::FRAMEBUFFER)
                );
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            Self {
                texture,
                framebuffer,
                width,
                height,
            }
        }
    }

    pub fn prepare_rendering(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Viewport(0, 0, self.width, self.height);
        }
    }

    pub fn export_png(&self, path: &str) {
        let mut download_buffer = vec![0u8; (self.width * self.height * 4) as usize];
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::GetTexImage(
                gl::TEXTURE_2D,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                download_buffer.as_mut_ptr().cast(),
            );
        }

        let file = std::fs::File::create(path).unwrap();
        let mut png = png::Encoder::new(file, self.width as _, self.height as _);
        png.set_color(png::ColorType::Rgba);
        png.set_depth(png::BitDepth::Eight);
        let mut writer = png.write_header().unwrap();
        writer
            .write_image_data(
                &download_buffer
                    .chunks((WINDOW_WIDTH * 4) as usize)
                    .rev()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        writer.finish().unwrap();
    }
}

 */

fn main() {}
