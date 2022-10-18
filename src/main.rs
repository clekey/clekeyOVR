extern crate core;

mod config;
mod global;
mod input_method;
mod ovr_controller;
mod utils;

use crate::config::CleKeyConfig;
use crate::input_method::IInputMethod;
use crate::ovr_controller::OVRController;
use crate::utils::Vec2;
use openvr::cstr;
use openvr::overlay::OwnedInVROverlay;
use sdl2::video::GLProfile;
use std::collections::VecDeque;
use skia_safe::gpu::{BackendRenderTarget, SurfaceOrigin};
use skia_safe::{ColorType, Paint, Surface};

const WINDOW_HEIGHT: u32 = 256;
const WINDOW_WIDTH: u32 = 512;

fn main() {
    // sdl initialization
    let sdl = sdl2::init().expect("sdl initialization error");
    let sdl_video = sdl.video().expect("sdl video");
    sdl_video.gl_attr().set_double_buffer(true);
    sdl_video.gl_attr().set_context_major_version(4);
    sdl_video.gl_attr().set_context_minor_version(1);
    sdl_video.gl_attr().set_context_profile(GLProfile::Core);

    let window = sdl_video
        .window("clekeyOVR", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position(0, 0)
        .opengl()
        .build()
        .expect("window creation");
    let sdl_gl_ctx = window.gl_create_context()
        .expect("sdl gl init");

    let mut skia_ctx = skia_safe::gpu::DirectContext::new_gl(None, None)
        .expect("skia gpu context creation");

    // debug block
    #[cfg(debug_assertions)]
    let mut window_surface = {
        window.gl_make_current(&sdl_gl_ctx).expect("sdl gl make current");
        // init gl context here
        let fbi;
        unsafe {
            gl::Viewport(0, 0, WINDOW_WIDTH as gl::types::GLsizei, WINDOW_HEIGHT as gl::types::GLsizei);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            let mut fboid: u32 = 0;
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid as *mut u32 as *mut i32);
            fbi = skia_safe::gpu::gl::FramebufferInfo {
                fboid,
                format: gl::RGBA8,
            };
        }
        let target = BackendRenderTarget::new_gl((WINDOW_WIDTH as _, WINDOW_HEIGHT as _), None, 8, fbi);
        Surface::from_backend_render_target(
            &mut skia_ctx, &target,
            SurfaceOrigin::BottomLeft, ColorType::RGBA8888, None, None)
            .expect("skia debug sufface creation")
    };

    // openvr initialization

    let ovr = openvr::init(openvr::ApplicationType::Overlay).expect("ovr");

    let overlay = ovr.overlay().expect("openvr overlay must be accessible");
    let input = ovr.input().expect("openvr input must be accessible");

    input
        .set_action_manifest_path(cstr!(r"C:\Users\anata\clekey-ovr-build\actions.json"))
        .expect("");

    let action_left_stick = input
        .get_action_handle(cstr!("/actions/input/in/left_stick"))
        .expect("action left_stick not found");
    let action_left_click = input
        .get_action_handle(cstr!("/actions/input/in/left_click"))
        .expect("action left_click not found");
    let action_left_haptic = input
        .get_action_handle(cstr!("/actions/input/in/left_haptic"))
        .expect("action left_haptic not found");
    let action_right_stick = input
        .get_action_handle(cstr!("/actions/input/in/right_stick"))
        .expect("action right_stick not found");
    let action_right_click = input
        .get_action_handle(cstr!("/actions/input/in/right_click"))
        .expect("action right_click not found");
    let action_right_haptic = input
        .get_action_handle(cstr!("/actions/input/in/right_haptic"))
        .expect("action right_haptic not found");
    let action_set_input = input.get_action_set_handle(cstr!("/actions/input"));

    let overlay_handle = OwnedInVROverlay::new(
        overlay,
        cstr!("com.anatawa12.clekey-ovr"),
        cstr!("clekey-ovr"),
    )
    .expect("create overlay");

    overlay_handle
        .set_overlay_width_in_meters(2.0)
        .expect("overlay");
    overlay_handle.set_overlay_alpha(1.0).expect("overlay");

    // gl main

    let canvas = window_surface.canvas();
    let paint = Paint::default();
    canvas.draw_rect(skia_safe::Rect::new(0.0, 0.0, 100.0, 100.0), &paint);
    window_surface.flush();
    //frame.clear_color();

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
            stick: [0.0, 0.0],
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
