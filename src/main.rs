mod config;
mod global;
mod graphics;
mod input_method;
mod os;
mod ovr_controller;
mod utils;

use crate::config::{load_config, CleKeyConfig};
use crate::graphics::{draw_center, draw_ring};
use crate::input_method::{HardKeyButton, IInputMethod, InputNextAction};
use crate::ovr_controller::{ActionSetKind, ButtonKind, OVRController, OverlayPlane};
use gl::types::GLuint;
use glam::{UVec2, Vec2};
use glfw::{Context, OpenGlProfileHint, WindowHint};
use skia_safe::font_style::{Slant, Weight, Width};
use skia_safe::gpu::gl::TextureInfo;
use skia_safe::gpu::{BackendTexture, Mipmapped, SurfaceOrigin};
use skia_safe::textlayout::FontCollection;
use skia_safe::{gpu, AlphaType, ColorType, FontMgr, FontStyle, Image, Surface};
#[cfg(feature = "debug_window")]
use skia_safe::{gpu::BackendRenderTarget, Rect, SamplingOptions};
use std::collections::VecDeque;
use std::ptr::null;
use std::rc::Rc;

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

    let mut skia_ctx = gpu::DirectContext::new_gl(None, None).expect("skia gpu context creation");

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
            fbi = gpu::gl::FramebufferInfo {
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

    let ovr_controller = OVRController::new(".".as_ref()).expect("ovr controller");
    ovr_controller
        .load_config(&config)
        .expect("loading config on ovr");

    let mut kbd = KeyboardManager::new(&ovr_controller, &config);

    let mut app = Application {
        ovr_controller: &ovr_controller,
        keyboard: &mut kbd,
        #[cfg(feature = "openvr")]
        status: Rc::new(Waiting),
        #[cfg(not(feature = "openvr"))]
        status: Rc::new(Inputting),
    };

    let font_mgr = FontMgr::new();
    let mut fonts = FontCollection::new();
    let mut font_families = Vec::new();

    for e in global::get_resources_dir()
        .join("fonts")
        .read_dir()
        .expect("read dir")
    {
        let e = e.expect("read dir");
        if e.path().extension() == Some("otf".as_ref())
            || e.path().extension() == Some("ttf".as_ref())
        {
            let face = font_mgr
                .new_from_data(&std::fs::read(e.path()).expect("read data"), None)
                .expect("new from data");
            font_families.push(face.family_name());
            println!("loaded: {:?}", face);
        }
    }
    println!("font_families: {:?}", font_families);

    // TODO: find way to use Noto Sans in rendering instead of system fonts
    fonts.set_default_font_manager(Some(font_mgr), None);
    println!(
        "find_typefaces: {:?}",
        fonts.find_typefaces(
            &font_families,
            FontStyle::new(Weight::MEDIUM, Width::NORMAL, Slant::Upright)
        )
    );

    // gl initialiation

    let mut left_ring = create_surface(&mut skia_ctx.clone().into(), WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut right_ring = create_surface(&mut skia_ctx.clone().into(), WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut center_field = create_surface(
        &mut skia_ctx.clone().into(),
        WINDOW_WIDTH,
        WINDOW_HEIGHT / 8,
    );

    //frame.clear_color();

    while !window.should_close() {
        glfw.poll_events();
        for (_, _) in glfw::flush_messages(&events) {}

        // TODO: openvr tick

        app.status.clone().tick(&mut app);

        ovr_controller.draw_if_visible(LeftRight::Left.into(), || {
            draw_ring(
                &app.keyboard.status,
                LeftRight::Left,
                true,
                &config.left_ring,
                &fonts,
                &font_families,
                &mut left_ring.surface,
            );
            left_ring.gl_tex_id
        });

        ovr_controller.draw_if_visible(LeftRight::Right.into(), || {
            draw_ring(
                &app.keyboard.status,
                LeftRight::Right,
                false,
                &config.right_ring,
                &fonts,
                &font_families,
                &mut right_ring.surface,
            );
            right_ring.gl_tex_id
        });

        ovr_controller.draw_if_visible(OverlayPlane::Center, || {
            draw_center(
                &app.keyboard.status,
                &config.completion,
                &fonts,
                &font_families,
                &mut center_field.surface,
            );
            center_field.gl_tex_id
        });

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
                    Rect::from_xywh(0.0, half_width, width, width / 8.0),
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
        app.ovr_controller
            .set_active_action_set([ActionSetKind::Waiting]);

        app.ovr_controller.hide_all_overlay();

        if app.ovr_controller.click_started(HardKeyButton::CloseButton) {
            app.status = Rc::new(Inputting);
        }
    }
}

struct Inputting;

impl ApplicationStatus for Inputting {
    fn tick(&self, app: &mut Application) {
        app.ovr_controller.set_active_action_set([
            ActionSetKind::Suspender,
            ActionSetKind::Input,
            ActionSetKind::Waiting,
        ]);
        app.ovr_controller.update_status(&mut app.keyboard.status);

        app.ovr_controller.show_overlay(OverlayPlane::Left);
        app.ovr_controller.show_overlay(OverlayPlane::Right);
        if !app.keyboard.status.buffer.is_empty() {
            app.ovr_controller.show_overlay(OverlayPlane::Center);
        } else {
            app.ovr_controller.hide_overlay(OverlayPlane::Center);
        }

        if app.keyboard.tick() {
            app.status = Rc::new(Waiting);
        }

        if app.ovr_controller.button_status(ButtonKind::SuspendInput) {
            app.status = Rc::new(Suspending)
        }
    }
}

struct Suspending;

impl ApplicationStatus for Suspending {
    fn tick(&self, app: &mut Application) {
        app.ovr_controller
            .set_active_action_set([ActionSetKind::Suspender]);
        app.ovr_controller.hide_all_overlay();
        if !app.ovr_controller.button_status(ButtonKind::SuspendInput) {
            app.status = Rc::new(Inputting)
        }
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
    buffer: String,
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
                buffer: String::new(),
            },
        }
    }

    pub(crate) fn tick(&mut self) -> bool {
        if (self.status.left.click_started() || self.status.right.click_started())
            && self.status.left.selection != -1
            && self.status.right.selection != -1
        {
            match (self.status.left.selection, self.status.right.selection) {
                (5, 6) => {
                    if self.status.buffer.is_empty() {
                        // close keyboard
                        return true;
                    } else {
                        // henkan key: nop currently
                    }
                }
                (5, 7) => {
                    if self.status.buffer.is_empty() {
                        // new line
                        os::enter_enter();
                    } else {
                        // kakutei key: just flush currently
                        self.flush();
                    }
                }
                (6, 6) => {
                    // backspace
                    if let Some(_) = self.status.buffer.pop() {
                        if self.status.buffer.is_empty() {
                            self.status.method.set_inputted_table();
                        }
                    } else {
                        os::enter_backspace();
                    }
                }
                (6, 7) => {
                    // space
                    if self.status.buffer.is_empty() {
                        os::enter_char(' ');
                    } else {
                        self.status.buffer.push(' ');
                    }
                }
                (7, 6) => self.move_to_next_plane(),
                (7, 7) => self.swap_sign_oplane(),
                (l @ 0..=7, r @ 0..=7) => {
                    let action = self
                        .status
                        .method
                        .on_input(UVec2::new(l as u32, r as u32), &mut self.status.buffer);
                    self.do_input_action(action)
                }
                (l, r) => unreachable!("{}, {}", l, r),
            }
        }

        for x in HardKeyButton::VALUES {
            if self.ovr_controller.click_started(x) {
                match x {
                    HardKeyButton::CloseButton => return true,
                    #[allow(unreachable_patterns)]
                    x => {
                        let action = self.status.method.on_hard_input(x);
                        self.do_input_action(action)
                    }
                }
            }
        }
        return false;
    }

    fn do_input_action(&mut self, action: InputNextAction) {
        match action {
            InputNextAction::Nop => (),
            InputNextAction::EnterChar(c) => os::enter_char(c),
        }
    }

    fn move_to_next_plane(&mut self) {
        if self.is_sign {
            // if current is sign, back to zero
            std::mem::swap(&mut self.sign_input, &mut self.status.method);
            self.is_sign = false
        }
        // rotate
        std::mem::swap(&mut self.status.method, self.methods.front_mut().unwrap());
        self.methods.rotate_left(1);
    }

    fn swap_sign_oplane(&mut self) {
        std::mem::swap(&mut self.sign_input, &mut self.status.method);
        self.is_sign = !self.is_sign;
    }

    pub fn flush(&mut self) {
        let buffer = std::mem::take(&mut self.status.buffer);
        if !buffer.is_empty() {
            os::copy_text_and_enter_paste_shortcut(&buffer);
        }
    }
}
