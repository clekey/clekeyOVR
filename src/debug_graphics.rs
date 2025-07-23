#![allow(unsafe_op_in_unsafe_fn)] // TODO

use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
use gl::types::{GLenum, GLint, GLsizei, GLuint};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use log::error;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::mem::size_of_val;
use std::num::NonZeroU32;
use std::ptr::null;
use std::time::Duration;
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowAttributes};

#[allow(unused)]
pub struct DebugWindow {
    pub raw_window_handle: RawWindowHandle,
    pub window: Window,
    pub event_loop: EventLoop<()>,
}

impl DebugWindow {
    pub fn new(width: f32, height: f32) -> DebugWindow {
        let event_loop =
            winit::event_loop::EventLoop::new().expect("Failed to create an event loop");
        let window_attributes =
            WindowAttributes::default().with_inner_size(LogicalSize::new(width, height));

        #[allow(deprecated)]
        let window = event_loop
            .create_window(window_attributes)
            .expect("Failed to create window");
        let raw_window_handle = window
            .window_handle()
            .expect("Failed to get window handle")
            .as_raw();

        DebugWindow {
            event_loop,
            window,
            raw_window_handle,
        }
    }

    pub fn with_surface<Config>(
        self,
        gl_config: &Config,
    ) -> DebugWindowWithSurface<<<Config as GetGlDisplay>::Target as GlDisplay>::WindowSurface>
    where
        Config: GlConfig,
        Config: GetGlDisplay,
        <Config as GetGlDisplay>::Target: GlDisplay<Config = Config>,
    {
        let size = self.window.inner_size();
        let w = size.width.try_into().unwrap();
        let h = size.height.try_into().unwrap();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::default().build(
            self.raw_window_handle,
            w,
            h,
        );
        let surface = unsafe {
            gl_config
                .display()
                .create_window_surface(gl_config, &attrs)
                .unwrap()
        };
        DebugWindowWithSurface {
            raw_window_handle: self.raw_window_handle,
            window: self.window,
            event_loop: self.event_loop,
            surface,
        }
    }
}

#[allow(unused)]
pub struct DebugWindowWithSurface<Surface: GlSurface<WindowSurface>> {
    pub raw_window_handle: RawWindowHandle,
    pub window: Window,
    pub surface: Surface,
    pub event_loop: EventLoop<()>,
}

impl<Surface: GlSurface<WindowSurface>> DebugWindowWithSurface<Surface> {
    pub fn pump_events(&mut self, mut handler: impl FnMut(Event<()>)) {
        #[allow(deprecated)]
        self.event_loop
            .pump_events(Some(Duration::ZERO), |e, _| handler(e));
    }
}

pub struct DebugRenderer {
    shader_program: GLuint,
    bottom_left_uniform: GLint,
    size_uniform: GLint,
    texture_uniform: GLint,

    vertex_buffer_object: GLuint,
}

fn check_err(at: &str) {
    unsafe {
        while let Some(err) = NonZeroU32::new(gl::GetError()) {
            error!("gl error: {}: 0x{:x}", at, err);
        }
    }
}

impl DebugRenderer {
    pub unsafe fn init() -> DebugRenderer {
        // region shader
        let vs = compile_shader(
            gl::VERTEX_SHADER,
            "#version 400\n\
            layout(location = 0) in vec2 pos;\n\
            uniform vec2 uBottomLeft;\n\
            uniform vec2 size;\n\
            out vec2 UV;\n\
            void main() {\n\
                UV = pos;\n\
                gl_Position.xy = UV * size + uBottomLeft;\n\
                gl_Position.zw = vec2(0, 1);\n\
            }\n",
        );
        let fs = compile_shader(
            gl::FRAGMENT_SHADER,
            "#version 400\n\
            in vec2 UV;\n\
            out vec3 color;\n\
            \n\
            uniform sampler2D rendered_texture;\n\
            \n\
            void main() {\n\
                color = texture(rendered_texture, UV).xyz;\n\
                //color = vec3(UV, 0);\n\
            }\n",
        );
        let shader_program = link_shader(&[fs, vs]);
        let pos_attrib = gl::GetAttribLocation(shader_program, "pos\0".as_ptr().cast());
        let bottom_left_uniform =
            gl::GetUniformLocation(shader_program, "uBottomLeft\0".as_ptr().cast());
        let size_uniform = gl::GetUniformLocation(shader_program, "size\0".as_ptr().cast());
        let texture_uniform =
            gl::GetUniformLocation(shader_program, "rendered_texture\0".as_ptr().cast());
        // endregion

        let points: [(f32, f32); 6] = [
            (1.0, 0.0),
            (0.0, 0.0),
            (0.0, 1.0),
            (0.0, 1.0),
            (1.0, 0.0),
            (1.0, 1.0),
        ];
        let mut vertex_buffer_object = 0;
        gl::GenBuffers(1, &mut vertex_buffer_object);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size_of_val(&points) as isize,
            points.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        let mut vertex_buffer_object = 0;
        gl::GenVertexArrays(1, &mut vertex_buffer_object);
        gl::BindVertexArray(vertex_buffer_object);
        gl::EnableVertexAttribArray(pos_attrib as _);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object);
        gl::VertexAttribPointer(pos_attrib as _, 2, gl::FLOAT, gl::FALSE, 0, null());

        check_err("end of init");

        DebugRenderer {
            shader_program,
            bottom_left_uniform,
            size_uniform,
            texture_uniform,
            vertex_buffer_object,
        }
    }

    pub unsafe fn draw(&self, left: GLuint, right: GLuint, center: GLuint) {
        // wipe the drawing surface clear
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        //gl::Disable(gl::BLEND);
        gl::Viewport(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        gl::UseProgram(self.shader_program);
        gl::BindVertexArray(self.vertex_buffer_object);

        self.draw_texture(left, -1.0, 0.0, 1.0, 1.0);
        self.draw_texture(right, 0.0, 0.0, 1.0, 1.0);
        self.draw_texture(center, -1.0, -1.0, 2.0, 1.0);

        check_err("drawing desktop gui");
    }

    unsafe fn draw_texture(&self, tex: GLuint, x: f32, y: f32, width: f32, height: f32) {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, tex);

        gl::Uniform1i(self.texture_uniform, 0);
        gl::Uniform2f(self.bottom_left_uniform, x, y);
        gl::Uniform2f(self.size_uniform, width, height);

        gl::DrawArrays(gl::TRIANGLES, 0, 6);
    }
}

unsafe fn compile_shader(type_: GLenum, script: &str) -> GLuint {
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
        error!(
            "compile error: (0x{:x}): {}",
            success,
            String::from_utf8_unchecked(std::mem::transmute(info))
        );
    }

    shader
}

unsafe fn link_shader(shaders: &[GLuint]) -> GLuint {
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
