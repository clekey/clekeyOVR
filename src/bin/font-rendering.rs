use crate::font_rendering::FontAtlas;
use font_kit::handle::Handle;
use gl::types::{GLenum, GLint, GLsizei, GLuint};
use glfw::{Context, OpenGlProfileHint, WindowHint};
use log::error;
use pathfinder_geometry::rect::RectI;
use pathfinder_geometry::vector::{Vector2F, Vector2I, vec2f, vec2i};
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

    for (i, canvas) in atlas.canvases().iter().enumerate() {
        let file = std::fs::File::create(format!("canvas.{}.png", i + 1)).unwrap();
        let mut png = png::Encoder::new(file, canvas.size.x() as u32, canvas.size.y() as u32);
        png.set_color(png::ColorType::Grayscale);
        png.set_depth(png::BitDepth::Eight);
        let mut writer = png.write_header().unwrap();
        writer.write_image_data(&canvas.pixels).unwrap();
        writer.finish().unwrap();
    }

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

        // shader
        let vs = compile_shader(
            gl::VERTEX_SHADER,
            "#version 400\n\
            layout(location = 0) in vec2 in_pos;\n\
            layout(location = 1) in vec3 in_uv_tex;\n\
            out vec3 v2f_uv_tex;\n\
            void main() {\n\
                v2f_uv_tex = in_uv_tex;\n\
                gl_Position.xy = in_pos;\n\
                gl_Position.zw = vec2(0, 1);\n\
            }\n",
        );
        let fs = compile_shader(
            gl::FRAGMENT_SHADER,
            "#version 400\n\
            // specify precision of sampler2darray\n\
            precision highp sampler2DArray;\n\
            \n\
            in vec3 v2f_uv_tex;\n\
            out vec4 color;\n\
            \n\
            uniform sampler2DArray font_textures;\n\
            //uniform sampler2D font_textures;\n\
            \n\
            void main() {\n\
                color.xyz = vec3(1.0);\n\
                color.w = texture(font_textures, v2f_uv_tex.xyz).r;\n\
            }\n",
        );
        let shader_program = link_shader(&[fs, vs]);
        let in_pos_attrib = gl::GetAttribLocation(shader_program, c"in_pos".as_ptr());
        let in_uv_tex_attrib = gl::GetAttribLocation(shader_program, c"in_uv_tex".as_ptr());
        let font_textures_attrib =
            gl::GetUniformLocation(shader_program, c"font_textures".as_ptr());
        println!("in_pos_attrib: {in_pos_attrib}");
        println!("in_uv_tex_attrib: {in_uv_tex_attrib}");
        println!("font_textures_attrib: {font_textures_attrib}");

        // upload font atlas
        let mut font_atlas_texture = 0;
        gl::GenTextures(1, &mut font_atlas_texture);
        gl::BindTexture(gl::TEXTURE_2D_ARRAY, font_atlas_texture);
        let array = atlas
            .canvases()
            .iter()
            .flat_map(|x| &x.pixels)
            .copied()
            .collect::<Vec<_>>();
        println!("array: {:?}", array.as_ptr());
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexImage3D(
            gl::TEXTURE_2D_ARRAY,
            0,
            gl::RGBA8 as _, //gl::R8 as _,
            atlas.canvas_size().x() as _,
            atlas.canvas_size().y() as _,
            atlas.canvases().len() as _,
            0,
            gl::RED,
            gl::UNSIGNED_BYTE,
            array.as_ptr().cast(),
        );
        drop(array);
        gl::TexParameteri(
            gl::TEXTURE_2D_ARRAY,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR_MIPMAP_LINEAR as _,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D_ARRAY,
            gl::TEXTURE_MAG_FILTER,
            gl::LINEAR_MIPMAP_LINEAR as _,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D_ARRAY,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as _,
        );
        gl::TexParameteri(
            gl::TEXTURE_2D_ARRAY,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as _,
        );
        gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAX_LEVEL, 0);
        gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);

        // attributes
        let text = "あいう".chars().collect::<Vec<_>>();
        let glyphs = text
            .iter()
            .map(|&c| font.glyph_for_char(c).unwrap())
            .collect::<Vec<_>>();
        let (glyph_info, update) = atlas
            .prepare_glyphs(&glyphs.iter().map(|&g| (&font, g)).collect::<Vec<_>>())
            .unwrap();
        assert!(!update, "No update should be performed");
        let uv_scale = Vector2F::splat(1.0) / atlas.canvas_size().to_f32();
        let pos_scale =
            Vector2F::splat(1.0) / Vector2I::new(WINDOW_WIDTH, WINDOW_HEIGHT).to_f32() * 0.5;

        let mut points = Vec::<[f32; 2]>::with_capacity(glyphs.len() * 6);
        let mut uv_tex = Vec::<([f32; 2], f32)>::with_capacity(glyphs.len() * 6);

        //let mut cursor = vec2f(0.0, 0.0);
        let mut cursor = vec2f(0.0, 0.5);
        for info in glyph_info {
            let advance = info.advance * pos_scale;

            let poly_rect =
                RectI::new(info.rasterize_offset, info.glyph_size).to_f32() * pos_scale + cursor;
            let uv_rect =
                RectI::new(info.glyph_origin, info.glyph_size * vec2i(1, -1)).to_f32() * uv_scale;

            points.push(poly_rect.upper_right().0.0);
            points.push(poly_rect.lower_left().0.0);
            points.push(poly_rect.origin().0.0);
            points.push(poly_rect.lower_left().0.0);
            points.push(poly_rect.upper_right().0.0);
            points.push(poly_rect.lower_right().0.0);

            let canvas = info.canvas_id as f32;
            uv_tex.push((uv_rect.upper_right().0.0, canvas));
            uv_tex.push((uv_rect.lower_left().0.0, canvas));
            uv_tex.push((uv_rect.origin().0.0, canvas));
            uv_tex.push((uv_rect.lower_left().0.0, canvas));
            uv_tex.push((uv_rect.upper_right().0.0, canvas));
            uv_tex.push((uv_rect.lower_right().0.0, canvas));

            cursor += advance;
        }

        // vbo: vertex buffer object
        // vao: vertex array object

        let mut points_vbo = 0;
        gl::GenBuffers(1, &mut points_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of_val::<[_]>(points.as_slice()) as isize,
            points.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        let mut uv_tex_vbo = 0;
        gl::GenBuffers(1, &mut uv_tex_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, uv_tex_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of_val::<[_]>(uv_tex.as_slice()) as isize,
            uv_tex.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        let mut points_vao = 0;
        gl::GenVertexArrays(1, &mut points_vao);
        gl::BindVertexArray(points_vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::EnableVertexAttribArray(in_pos_attrib as _);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, null());

        gl::BindBuffer(gl::ARRAY_BUFFER, uv_tex_vbo);
        gl::EnableVertexAttribArray(in_uv_tex_attrib as _);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, null());

        // rendering
        gl::BindFramebuffer(gl::FRAMEBUFFER, gl_framebuffer);
        gl::Viewport(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);

        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::UseProgram(shader_program);

        gl::BindVertexArray(points_vao);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D_ARRAY, font_atlas_texture);
        gl::Uniform1i(font_textures_attrib, 0);

        gl::DrawArrays(gl::TRIANGLES, 0, points.len() as i32);
        gl::Disable(gl::BLEND);

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
}

unsafe fn compile_shader(type_: GLenum, script: &str) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(type_);
        gl::ShaderSource(
            shader,
            1,
            &script.as_ptr().cast::<i8>(),
            &(script.len() as GLint),
        );
        gl::CompileShader(shader);

        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut info = Vec::new();
            let mut len: GLsizei = 512;
            while info.capacity() < (len as usize) {
                info.reserve(len as usize);
                gl::GetShaderInfoLog(shader, info.capacity() as _, &mut len, info.as_mut_ptr());
            }
            info.set_len(len as usize);
            panic!(
                "compile error: (0x{:x}): {}",
                success,
                String::from_utf8_unchecked(std::mem::transmute(info))
            );
        }

        shader
    }
}

unsafe fn link_shader(shaders: &[GLuint]) -> GLuint {
    unsafe {
        let shader_program = gl::CreateProgram();
        for shader in shaders {
            gl::AttachShader(shader_program, *shader);
        }
        gl::LinkProgram(shader_program);

        let mut success = 0;
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);

        if success == 0 {
            let mut info = Vec::new();
            let mut len: GLsizei = 512;
            while info.capacity() < (len as usize) {
                info.reserve(len as usize);
                gl::GetProgramInfoLog(
                    shader_program,
                    info.capacity() as _,
                    &mut len,
                    info.as_mut_ptr(),
                );
            }
            info.set_len(len as usize);
            error!(
                "link error: (0x{:x}): {}",
                success,
                String::from_utf8_unchecked(std::mem::transmute(info))
            );
        }

        shader_program
    }
}
