use crate::font_rendering::{FontAtlas, FontRenderer};
use font_kit::handle::Handle;
use gl::types::{GLenum, GLint, GLsizei, GLuint};
use glfw::{Context, OpenGlProfileHint, WindowHint};
use log::error;
use pathfinder_color::ColorF;
use pathfinder_geometry::transform2d::{Matrix2x2F, Transform2F};
use pathfinder_geometry::vector::{Vector2F, Vector2I, vec2f};
use std::ptr::null;
use std::sync::Arc;

#[path = "../font_rendering.rs"]
mod font_rendering;

const WINDOW_HEIGHT: i32 = 1024;
const WINDOW_WIDTH: i32 = 1024;

fn main() {
    let mut atlas = FontAtlas::new(200.0, 65536, 16);

    let font = Arc::new(
        Handle::from_memory(
            Arc::new(Vec::from(include_bytes!(
                "../../resources/fonts/NotoSansJP-Medium.otf"
            ))),
            0,
        )
        .load()
        .unwrap(),
    );

    let (glyphs, changed) = atlas
        .prepare_glyphs(&[
            (&font, font.glyph_for_char('あ').unwrap()),
            (&font, font.glyph_for_char('い').unwrap()),
            (&font, font.glyph_for_char('う').unwrap()),
        ])
        .unwrap();

    assert!(changed);
    println!("{glyphs:#?}");

    let (_, changed) = atlas
        .prepare_glyphs(&[
            (&font, font.glyph_for_char('あ').unwrap()),
            (&font, font.glyph_for_char('い').unwrap()),
            (&font, font.glyph_for_char('う').unwrap()),
        ])
        .unwrap();
    assert!(!changed);

    let (_, changed) = atlas
        .prepare_glyphs(&[
            (&font, font.glyph_for_char('え').unwrap()),
            (&font, font.glyph_for_char('お').unwrap()),
            (&font, font.glyph_for_char('か').unwrap()),
            (&font, font.glyph_for_char('が').unwrap()),
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

    let (mut window, events) = glfw
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
    let loader = |s: &str| glfw.get_proc_address_raw(s);
    gl::load_with(loader);

    unsafe {
        window.make_current();

        // note: opengl coordinate starts at left bottom corner as 0, 0 and increase 1, 1 for right top

        // generate framebuffer with texture
        let mut target_texture = 0;
        gl::GenTextures(1, &mut target_texture);
        gl::BindTexture(gl::TEXTURE_2D, target_texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as _,
            WINDOW_WIDTH as _,
            WINDOW_HEIGHT as _,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);

        let mut gl_framebuffer = 0;

        gl::GenFramebuffers(1, &mut gl_framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, gl_framebuffer);

        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, target_texture, 0);
        gl::DrawBuffers(1, [gl::COLOR_ATTACHMENT0].as_ptr());

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!(
                "Framebuffer rendering failed: {:x}",
                gl::CheckFramebufferStatus(gl::FRAMEBUFFER)
            );
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        let mut font_renderer = FontRenderer::new();
        font_renderer.update_texture(&atlas);

        // rendering
        gl::BindFramebuffer(gl::FRAMEBUFFER, gl_framebuffer);
        gl::Viewport(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);

        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        let color = ColorF::new(1.0, 0.0, 0.0, 0.5);
        let pos_scale =
            Vector2F::splat(1.0) / Vector2I::new(WINDOW_WIDTH, WINDOW_HEIGHT).to_f32() * 0.125;
        let angle = -0.0 * std::f32::consts::PI / 180.0;
        let matrix = Matrix2x2F::from_scale(pos_scale) * Matrix2x2F::from_rotation(angle);

        let regular_use_ideographs = include_str!("regular_use_utf8.txt");
        let mut cursor = vec2f(-1.0, 0.975);
        for text in [
            (0x3041..=0x3092)
                .map(|x| char::from_u32(x as u32).unwrap())
                .collect::<String>()
                .as_str(),
            (0x3093..=0x3094)
                .map(|x| char::from_u32(x as u32).unwrap())
                .collect::<String>()
                .as_str(),
        ]
        .into_iter()
        .chain((0..52).map(|i| &regular_use_ideographs[i * 3 * 40..][..3 * 40]))
        {
            font_renderer.draw_text_simple(
                &mut atlas,
                &font,
                color,
                Transform2F {
                    matrix,
                    vector: cursor,
                },
                text,
            );
            cursor -= matrix * vec2f(0.0, atlas.font_em_size());
        }

        gl::Flush();

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        window.swap_buffers();

        let mut download_buffer = vec![0u8; (WINDOW_WIDTH * WINDOW_HEIGHT * 4) as usize];
        gl::BindTexture(gl::TEXTURE_2D, target_texture);
        gl::GetTexImage(
            gl::TEXTURE_2D,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            download_buffer.as_mut_ptr().cast(),
        );

        {
            let file = std::fs::File::create("canvas.png").unwrap();
            let mut png = png::Encoder::new(file, WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32);
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

    for (i, canvas) in atlas.canvases().iter().enumerate() {
        let file = std::fs::File::create(format!("canvas.{}.png", i + 1)).unwrap();
        let mut png = png::Encoder::new(file, canvas.size.x() as u32, canvas.size.y() as u32);
        png.set_color(png::ColorType::Grayscale);
        png.set_depth(png::BitDepth::Eight);
        let mut writer = png.write_header().unwrap();
        writer.write_image_data(&canvas.pixels).unwrap();
        writer.finish().unwrap();
    }
}
