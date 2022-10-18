extern crate core;

mod config;
mod global;
mod graphics;
mod input_method;
#[cfg_attr(
    not(all(feature = "openvr", windows)),
    path = "ovr_controller.no-ovr.rs"
)]
mod ovr_controller;
mod utils;

use crate::config::{load_config, CleKeyConfig};
use crate::graphics::draw_ring;
use crate::input_method::IInputMethod;
use crate::ovr_controller::OVRController;
use crate::utils::Vec2;
use glfw::{Context, OpenGlProfileHint, WindowHint};
use skia_safe::gpu::{BackendRenderTarget, SurfaceOrigin};
use skia_safe::{ColorType, Paint, Surface};
use std::collections::VecDeque;

const WINDOW_HEIGHT: u32 = 1024;
const WINDOW_WIDTH: u32 = 1024;

#[derive(Copy, Clone)]
pub enum LeftRight {
    Left,
    Right,
}

fn main() {
    // glfw initialization
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::DoubleBuffer(true));
    glfw.window_hint(WindowHint::ContextVersionMajor(4));
    glfw.window_hint(WindowHint::ContextVersionMinor(1));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::Resizable(false));
    glfw.window_hint(WindowHint::CocoaRetinaFramebuffer(false));

    let (mut window, events) = glfw
        .create_window(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            "clekeyOVR",
            glfw::WindowMode::Windowed,
        )
        .expect("window creation");
    window.make_current();

    // gl crate initialization
    gl::load_with(|s| glfw.get_proc_address_raw(s));

    let mut skia_ctx =
        skia_safe::gpu::DirectContext::new_gl(None, None).expect("skia gpu context creation");

    // debug block
    #[cfg(feature = "debug_window")]
    let mut window_surface = {
        window.make_current();
        // init gl context here
        let fbi;
        unsafe {
            gl::Viewport(
                0,
                0,
                WINDOW_WIDTH as gl::types::GLsizei,
                WINDOW_HEIGHT as gl::types::GLsizei,
            );
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            let mut fboid: u32 = 0;
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid as *mut u32 as *mut i32);
            fbi = skia_safe::gpu::gl::FramebufferInfo {
                fboid,
                format: gl::RGBA8,
            };
        }
        let target =
            BackendRenderTarget::new_gl((WINDOW_WIDTH as _, WINDOW_HEIGHT as _), None, 8, fbi);
        Surface::from_backend_render_target(
            &mut skia_ctx,
            &target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        )
        .expect("skia debug sufface creation")
    };

    // openvr initialization

    let mut config = CleKeyConfig::default();

    load_config(&mut config);

    let ovr_controller = OVRController::new(".".as_ref()).expect("ovr controller");

    let kbd = KeyboardManager::new(&ovr_controller, &config);

    // gl main

    let canvas = window_surface.canvas();
    let paint = Paint::default();
    canvas.draw_rect(skia_safe::Rect::new(0.0, 0.0, 100.0, 100.0), &paint);
    window_surface.flush();
    //frame.clear_color();

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {}
        #[cfg(feature = "debug_window")]
        {
            draw_ring(
                &kbd.status,
                LeftRight::Left,
                true,
                &config.left_ring,
                &mut window_surface,
            );
            window_surface.flush();
            window.swap_buffers();
        }
    }
    println!("Hello, world!");
}

pub struct HandInfo {
    stick: Vec2,
    selection: i8,
    selection_old: i8,

    clicking: bool,
    clicking_old: bool,
}

impl HandInfo {
    pub fn new() -> Self {
        Self {
            stick: (0.0, 0.0),
            selection: -1,
            selection_old: -1,
            clicking: false,
            clicking_old: false,
        }
    }

    fn click_started(&self) -> bool {
        return self.clicking && !self.clicking_old;
    }
}

pub struct KeyboardStatus {
    left: HandInfo,
    right: HandInfo,
    method: Box<dyn IInputMethod>,
}

impl KeyboardStatus {
    pub fn get_selecting(&self, lr: LeftRight) -> (i8, i8) {
        match lr {
            LeftRight::Left => (self.left.selection, self.right.selection),
            LeftRight::Right => (self.right.selection, self.left.selection),
        }
    }

    fn stick_pos(&self, lr: LeftRight) -> Vec2 {
        match lr {
            LeftRight::Left => self.left.stick,
            LeftRight::Right => self.right.stick,
        }
    }
}

struct KeyboardManager<'ovr> {
    ovr_controller: &'ovr OVRController,
    sign_input: Box<dyn IInputMethod>,
    methods: VecDeque<Box<dyn IInputMethod>>,
    is_sign: bool,
    status: KeyboardStatus,
}

impl<'ovr> KeyboardManager<'ovr> {
    pub fn new(ovr: &'ovr OVRController, config: &CleKeyConfig) -> Self {
        use input_method::*;
        Self {
            ovr_controller: ovr,
            sign_input: Box::new(SignsInput::new()),
            methods: VecDeque::from([Box::new(EnglishInput::new()) as Box<dyn IInputMethod>]),
            is_sign: false,
            status: KeyboardStatus {
                left: HandInfo::new(),
                right: HandInfo::new(),
                method: Box::new(JapaneseInput::new()),
            },
        }
    }
}
