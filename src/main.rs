extern crate core;

mod config;
mod global;
mod graphics;
mod input_method;
mod ovr_controller;
mod utils;

use crate::config::{load_config, CleKeyConfig};
use crate::graphics::draw_ring;
use crate::input_method::{HardKeyButton, IInputMethod, InputNextAction, InputNextMoreAction};
use crate::ovr_controller::{ActionSetKind, ButtonKind, OverlayPlane, OVRController};
use gl::types::GLuint;
use glam::{UVec2, Vec2};
use glfw::{Context, OpenGlProfileHint, WindowHint};
use skia_safe::gpu::gl::{Format, TextureInfo};
use skia_safe::gpu::{BackendRenderTarget, BackendTexture, Mipmapped, SurfaceOrigin};
use skia_safe::{gpu, AlphaType, ColorType, Image, Paint, Rect, SamplingOptions, Surface};
use std::collections::VecDeque;
use std::os::macos::raw::stat;
use std::ptr::null;
use std::rc::Rc;
use crate::ButtonKind::SuspendInput;

const WINDOW_HEIGHT: i32 = 1024;
const WINDOW_WIDTH: i32 = 1024;

#[derive(Copy, Clone)]
pub enum LeftRight {
    Left = 0,
    Right = 1,
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
            WINDOW_WIDTH as _,
            WINDOW_HEIGHT as _,
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
            gl::Viewport(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            let mut fboid: u32 = 0;
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid as *mut u32 as *mut i32);
            fbi = skia_safe::gpu::gl::FramebufferInfo {
                fboid,
                format: gl::RGBA8,
            };
        }
        let target = BackendRenderTarget::new_gl((WINDOW_WIDTH, WINDOW_HEIGHT), None, 8, fbi);
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
    println!("{:?}", config);
    println!("{:#?}", config);

    let ovr_controller = OVRController::new(".".as_ref()).expect("ovr controller");
    ovr_controller.load_config(&config).expect("loading config on ovr");

    let mut kbd = KeyboardManager::new(&ovr_controller, &config);

    let mut app = Application {
        ovr_controller: &ovr_controller,
        keyboard: &mut kbd,
        status: Rc::new(Waiting),
    };

    // gl initialiation

    let mut left_ring = create_surface(&mut skia_ctx.clone().into(), WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut right_ring = create_surface(&mut skia_ctx.clone().into(), WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut center_field = create_surface(&mut skia_ctx.clone().into(), WINDOW_WIDTH, WINDOW_HEIGHT / 8);

    //frame.clear_color();

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {}

        // TODO: openvr tick

        app.status.clone().tick(&mut app);

        ovr_controller.draw_if_visible(LeftRight::Left.into(), || {
            draw_ring(
                &app.keyboard.status,
                LeftRight::Left,
                true,
                &config.left_ring,
                &mut left_ring.surface,
            );
            left_ring.gl_tex_id
        }).expect("drawing / updating left");

        ovr_controller.draw_if_visible(LeftRight::Right.into(), || {
            draw_ring(
                &app.keyboard.status,
                LeftRight::Right,
                true,
                &config.right_ring,
                &mut right_ring.surface,
            );
            right_ring.gl_tex_id
        }).expect("drawing / updating right");

        ovr_controller.draw_if_visible(OverlayPlane::Center, || {
            // TODO rendering
            center_field.gl_tex_id
        }).expect("drawing / updating center");

        #[cfg(feature = "debug_window")]
        {
            let canvas = window_surface.canvas();
            let width = WINDOW_WIDTH as f32;
            let half_width = width / 2.0;
            canvas
                .draw_image_rect_with_sampling_options(
                    &left_ring.image,
                    None,
                    Rect::from_xywh(0.0, 0.0, half_width, half_width),
                    SamplingOptions::default(),
                    &Default::default(),
                )
                .draw_image_rect_with_sampling_options(
                    &right_ring.image,
                    None,
                    Rect::from_xywh(half_width, 0.0, half_width, half_width),
                    SamplingOptions::default(),
                    &Default::default(),
                )
                .draw_image_rect_with_sampling_options(
                    &center_field.image,
                    None,
                    Rect::from_xywh(half_width, 0.0, width, width / 8.0),
                    SamplingOptions::default(),
                    &Default::default(),
                );
            window_surface.flush();
        }

        #[cfg(feature = "debug_window")]
        window.swap_buffers();
    }
    println!("Hello, world!");
}

struct Application<'a> {
    ovr_controller: &'a OVRController,
    keyboard: &'a mut KeyboardManager<'a>,
    status: Rc<dyn ApplicationStatus>,
}

trait ApplicationStatus {
    fn tick(&self, app: &mut Application);
}

struct Waiting;

impl ApplicationStatus for Waiting {
    fn tick(&self, app: &mut Application) {
        app.ovr_controller.set_active_action_set([ActionSetKind::Waiting])
            .expect("setting active action set");

        app.ovr_controller.hide_all_overlay().expect("hiding overlay");

        if app.ovr_controller.click_started(HardKeyButton::CloseButton) {
            app.status = Rc::new(Inputting);
        }
    }
}

struct Inputting;

impl ApplicationStatus for Inputting {
    fn tick(&self, app: &mut Application) {
        app.ovr_controller.set_active_action_set([ActionSetKind::Suspender, ActionSetKind::Input, ActionSetKind::Waiting]).expect("set_active_action_set");
        app.ovr_controller.update_status(&mut app.keyboard.status).expect("updating");

        app.ovr_controller.show_overlay(OverlayPlane::Left).expect("show overlay");
        app.ovr_controller.show_overlay(OverlayPlane::Right).expect("show overlay");
        if !app.keyboard.status.method.buffer().is_empty() {
            app.ovr_controller.show_overlay(OverlayPlane::Center).expect("show overlay");
        } else {
            app.ovr_controller.hide_overlay(OverlayPlane::Center).expect("show overlay");
        }

        if app.keyboard.tick() {
            app.status = Rc::new(Waiting);
        }

        if app.ovr_controller.button_status(SuspendInput) {
            app.status = Rc::new(Suspending)
        }
    }
}

struct Suspending;

impl ApplicationStatus for Suspending {
    fn tick(&self, app: &mut Application) {
    }
}

struct SurfaceInfo {
    gl_tex_id: GLuint,
    surface: Surface,
    image: Image,
}

fn create_surface(context: &mut gpu::RecordingContext, width: i32, height: i32) -> SurfaceInfo {
    let mut gl_tex_id = 0;
    unsafe {
        gl::GenTextures(1, &mut gl_tex_id);
        gl::BindTexture(gl::TEXTURE_2D, gl_tex_id);
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
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
    }

    let backend_texture = unsafe {
        BackendTexture::new_gl(
            (width, height),
            Mipmapped::No,
            TextureInfo {
                target: gl::TEXTURE_2D,
                format: gl::RGBA8,
                id: gl_tex_id,
            },
        )
    };
    let surface = Surface::from_backend_texture(
        context,
        &backend_texture,
        SurfaceOrigin::BottomLeft,
        None,
        ColorType::RGBA8888,
        None,
        None,
    )
    .expect("creating surface");
    let image = Image::from_texture(
        context,
        &backend_texture,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        AlphaType::Opaque,
        None,
    )
    .expect("image creation");

    SurfaceInfo {
        gl_tex_id,
        surface,
        image,
    }
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
            stick: Vec2::new(0.0, 0.0),
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
    pub fn new(ovr: &'ovr OVRController, _config: &CleKeyConfig) -> Self {
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

    pub(crate) fn tick(&mut self) -> bool {
        if self.status.left.click_started() || self.status.right.click_started()
            && self.status.left.selection != -1 && self.status.right.selection != -1 {
            if self.do_input_action(self.status.method.on_input(UVec2::new(self.status.left.selection as u32, self.status.right.selection as u32))) {
                return true;
            }
        }

        for x in HardKeyButton::VALUES {
            if self.ovr_controller.click_started(x) {
                if self.do_input_action(self.status.method.on_hard_input(x)) {
                    return true;
                }
            }
        }
        return false;
    }

    fn do_input_action(&mut self, action: InputNextAction) -> bool {
        if action.flush() {
            self.flush()
        }

        match action.action() {
            InputNextMoreAction::Nop => false,
            InputNextMoreAction::MoveToNextPlane => {
                if self.is_sign {
                    // if current is sign, back to zero
                    std::mem::swap(&mut self.sign_input, &mut self.status.method);
                }
                // rotate
                std::mem::swap(&mut self.status.method, self.methods.front_mut().unwrap());
                self.methods.rotate_left(1);
                false
            }
            InputNextMoreAction::MoveToSignPlane => {
                std::mem::swap(&mut self.sign_input, &mut self.status.method);
                false
            }
            InputNextMoreAction::EnterChar(c) => {
                // TODO: enter an char
                false
            }
            InputNextMoreAction::RemoveLastChar => {
                // TODO: enter delete char
                false
            }
            InputNextMoreAction::CloseKeyboard => {
                true
            }
            InputNextMoreAction::NewLine => {
                // TODO: enter 'enter' key
                false
            }
        }
    }

    pub fn flush(&mut self) {
        let buffer = self.status.method.get_and_clear_buffer();
        if !buffer.is_empty() {
            // TODO: flush
        }
    }
}
