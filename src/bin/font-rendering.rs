use crate::font_rendering::{FontAtlas, FontRenderer};
use font_kit::handle::Handle;
use gl::types::{GLsizei, GLuint};
use glfw::{Context, OpenGlProfileHint, WindowHint};
use pathfinder_color::ColorF;
use pathfinder_geometry::transform2d::{Matrix2x2F, Transform2F};
use pathfinder_geometry::vector::{Vector2F, Vector2I, vec2f};
use std::ptr::null;
use std::sync::Arc;
use std::time::Instant;

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
    let loader = |s: &str| glfw.get_proc_address_raw(s);
    gl::load_with(loader);

    unsafe {
        window.make_current();

        let regular_use_ideographs = include_str!("regular_use_utf8.txt");
        let hiragana = [
            (0x3041..=0x3092)
                .map(|x| char::from_u32(x as u32).unwrap())
                .collect::<String>(),
            (0x3093..=0x3094)
                .map(|x| char::from_u32(x as u32).unwrap())
                .collect::<String>(),
        ];
        let chars = (hiragana.iter().map(|x| x.as_str()))
            .chain((0..52).map(|i| &regular_use_ideographs[i * 3 * 40..][..3 * 40]))
            .collect::<Vec<_>>();

        // note: opengl coordinate starts at left bottom corner as 0, 0 and increase 1, 1 for right top

        let render_target0 = RenderTargetTexture::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let render_target1 = RenderTargetTexture::new(WINDOW_WIDTH, WINDOW_HEIGHT);
        let render_target2 = RenderTargetTexture::new(WINDOW_WIDTH, WINDOW_HEIGHT);

        let mut font_renderer = FontRenderer::new();
        font_renderer.update_texture(&atlas);

        let pos_scale =
            Vector2F::splat(1.0) / Vector2I::new(WINDOW_WIDTH, WINDOW_HEIGHT).to_f32() * 0.125;
        let angle = -0.0 * std::f32::consts::PI / 180.0;
        let matrix = Matrix2x2F::from_scale(pos_scale) * Matrix2x2F::from_rotation(angle);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);

        // rendering
        let render0_start = Instant::now();
        render_target0.prepare_rendering();
        gl::Clear(gl::COLOR_BUFFER_BIT);

        let mut cursor = vec2f(-1.0, 0.975);
        for text in chars.as_slice() {
            font_renderer
                .draw_text_simple(
                    &mut atlas,
                    &font,
                    ColorF::white(),
                    Transform2F {
                        matrix,
                        vector: cursor,
                    },
                    text,
                )
                .unwrap();
            cursor -= matrix * vec2f(0.0, atlas.font_em_size());
        }
        let render0_end = Instant::now();
        render_target0.export_png("canvas0.png");

        println!("rendering 0 took {:?}", render0_end - render0_start);

        let render1_start = Instant::now();
        render_target1.prepare_rendering();
        gl::Clear(gl::COLOR_BUFFER_BIT);

        let mut cursor = vec2f(-1.0, 0.975);
        for text in chars.as_slice() {
            font_renderer
                .draw_text_simple(
                    &mut atlas,
                    &font,
                    ColorF::new(1.0, 0.0, 0.0, 1.0),
                    Transform2F {
                        matrix,
                        vector: cursor,
                    },
                    text,
                )
                .unwrap();
            cursor -= matrix * vec2f(0.0, atlas.font_em_size());
        }
        let render1_end = Instant::now();
        println!("rendering 1 took {:?}", render1_end - render1_start);
        render_target1.export_png("canvas1.png");

        let render2_start = Instant::now();
        render_target2.prepare_rendering();
        gl::Clear(gl::COLOR_BUFFER_BIT);

        let mut cursor0 = vec2f(-1.0, 0.975);
        let mut info_transforms = Vec::with_capacity(chars.iter().map(|x| x.chars().count()).sum());
        for text in chars.as_slice() {
            let mut cursor = cursor0;
            let glyphs = text
                .chars()
                .map(|c| font.glyph_for_char(c).unwrap())
                .collect::<Vec<_>>();
            let (glyph_info, update) = atlas
                .prepare_glyphs(&glyphs.iter().map(|&g| (&font, g)).collect::<Vec<_>>())
                .unwrap();
            assert!(!update);
            info_transforms.extend(glyph_info.iter().map(|info| {
                let advance = matrix * info.advance;
                let transform = Transform2F {
                    matrix,
                    vector: cursor,
                };
                cursor += advance;
                (*info, transform)
            }));
            cursor0 -= matrix * vec2f(0.0, atlas.font_em_size());
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
