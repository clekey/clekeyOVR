#![allow(double_negations)] // false positive. see https://github.com/rust-lang/rust/issues/143980

#[macro_use]
mod utils;
mod config;
#[cfg(feature = "debug_window")]
mod debug_graphics;
mod font_rendering;
mod gl_primitives;
mod global;
mod graphics;
mod input_method;
mod licenses;
mod os;
mod ovr_controller;
mod resources;

use crate::config::{CleKeyConfig, UIMode, load_config};
use crate::graphics::GraphicsContext;
use crate::input_method::{CleKeyButton, CleKeyInputTable, HardKeyButton, InputNextAction};
use crate::ovr_controller::{ActionSetKind, ButtonKind, OVRController, OverlayPlane};
use crate::utils::GlContextExt;
use gl::types::GLuint;
use glam::Vec2;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin_winit::GlWindow;
use log::info;
use raw_window_handle::HasWindowHandle;
use std::collections::VecDeque;
use std::ffi::CString;
use std::mem::take;
use std::ptr::null;
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use winit::dpi::LogicalSize;
use winit::window::WindowAttributes;

const WINDOW_HEIGHT: i32 = 1024;
const WINDOW_WIDTH: i32 = 1024;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum LeftRight {
    Left = 0,
    Right = 1,
}

fn main() {
    simple_logger::init().unwrap();
    licenses::check_and_print_exit();
    info!("clekeyOVR version {}", env!("CARGO_PKG_VERSION"));
    info!("features: ");
    macro_rules! feature_log {
        ($feat: literal) => {
            info!(
                "  {}: {}",
                $feat,
                if cfg!(feature = $feat) {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        };
    }
    feature_log!("openvr");
    feature_log!("debug_window");
    feature_log!("debug_control");

    // resource initialization
    resources::init();

    // glut and winit
    #[allow(unused_mut)]
    let mut event_loop =
        winit::event_loop::EventLoop::new().expect("Failed to create an event loop");

    let mut display_builder = glutin_winit::DisplayBuilder::new();

    if cfg!(feature = "debug_window") || cfg!(windows) {
        let mut window_attributes = WindowAttributes::default();

        window_attributes =
            window_attributes.with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        if !cfg!(feature = "debug_window") {
            window_attributes = window_attributes.with_visible(false);
        }

        display_builder = display_builder.with_window_attributes(Some(window_attributes));
    }

    let (window, gl_config) = display_builder
        .build(&event_loop, ConfigTemplateBuilder::new(), |mut cfgs| {
            cfgs.next().unwrap()
        })
        .expect("creating window");
    #[allow(unused_mut)]
    let mut raw_window_handle = window
        .as_ref()
        .and_then(|w| w.window_handle().ok())
        .map(|h| h.as_raw());

    // glutin
    let gl_display;
    let gl_context;
    let gl_surface;
    unsafe {
        gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version::new(
                4, 1,
            ))))
            .build(raw_window_handle);
        gl_context = gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap();
        gl_surface = window.as_ref().map(|window| {
            let attrs = window.build_surface_attributes(Default::default()).unwrap();
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .expect("failed to create surface")
        })
    }

    #[allow(unused_variables)]
    let gl_context = if let Some(ref gl_surface) = gl_surface {
        gl_context
            .make_current(gl_surface)
            .expect("creating context")
    } else {
        gl_context
            .make_current_surfaceless()
            .expect("creating context")
    };

    // gl crate initialization
    gl::load_with(|s| gl_display.get_proc_address(&CString::new(s).unwrap()));

    let mut graphics_context = GraphicsContext::new();

    // debug block
    #[cfg(feature = "debug_window")]
    let debug_renderer = unsafe { debug_graphics::DebugRenderer::init() };

    // openvr initialization

    let mut config = CleKeyConfig::default();

    load_config(&mut config);

    let ovr_controller = OVRController::new(&global::get_resources_dir()).expect("ovr controller");
    ovr_controller
        .load_config(&config)
        .expect("loading config on ovr");

    let mut app = Application::new(
        &ovr_controller,
        &config,
        if cfg!(feature = "openvr") {
            Rc::new(Waiting)
        } else {
            Rc::new(Inputting)
        },
        Surfaces {
            left_ring: create_surface(WINDOW_WIDTH, WINDOW_HEIGHT),
            right_ring: create_surface(WINDOW_WIDTH, WINDOW_HEIGHT),
            center_field: create_surface(WINDOW_WIDTH, WINDOW_HEIGHT / 2),
        },
    );

    // gl initialiation

    app.set_default_renderers();

    //frame.clear_color();

    let mut fps_calc = utils::FPSComputer::<30>::new();

    // fps throttling
    let frame_duration = {
        let actual_fps = config.fps.max(1.0);
        let frame_dur_nano = Duration::new(1, 0).as_nanos() as f64 / actual_fps as f64;
        Duration::new(0, frame_dur_nano as u32)
    };

    loop {
        let frame_end_expected = Instant::now() + frame_duration;

        #[allow(deprecated)]
        winit::platform::pump_events::EventLoopExtPumpEvents::pump_events(
            &mut event_loop,
            Some(Duration::ZERO),
            |_e, _active| {
                #[cfg(feature = "debug_control")]
                ovr_controller.accept_debug_control(_e)
            },
        );

        graphics_context.receive_atlas();

        // TODO: openvr tick

        app.app_status.clone().tick(&mut app);

        // Surface::flush() does not work as expect but the following is working.
        //  Surface::image_snapshot(<surface>).backend_texture(true);
        // I don't know why this is working but It's working so I'm using that.

        ovr_controller.draw_if_visible(LeftRight::Left.into(), || {
            app.surfaces.left_ring.render(&mut graphics_context, &app)
        });

        ovr_controller.draw_if_visible(LeftRight::Right.into(), || {
            app.surfaces.right_ring.render(&mut graphics_context, &app)
        });

        ovr_controller.draw_if_visible(OverlayPlane::Center, || {
            app.surfaces
                .center_field
                .render(&mut graphics_context, &app)
        });

        #[cfg(feature = "debug_window")]
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            debug_renderer.draw(
                app.surfaces.left_ring.gl_tex_id,
                app.surfaces.right_ring.gl_tex_id,
                app.surfaces.center_field.gl_tex_id,
            );
            gl::Flush();

            gl_surface
                .as_ref()
                .unwrap()
                .swap_buffers(&gl_context)
                .expect("Swap buffers");
        }
        let (average, one_frame) = fps_calc.on_frame();
        print!("FPS: {average:7.3} fps ({one_frame:7.3} fps in short period)\r");
        // sleep for next frame.
        sleep(frame_end_expected.duration_since(Instant::now()));
    }
}

struct Application<'a> {
    ovr_controller: &'a OVRController,
    sign_input: &'static CleKeyInputTable<'static>,
    methods: VecDeque<&'static CleKeyInputTable<'static>>,
    is_sign: bool,
    kbd_status: KeyboardStatus,
    click_started: Instant,
    app_status: Rc<dyn ApplicationStatus>,
    config: &'a CleKeyConfig,
    surfaces: Surfaces,
}

impl<'a> Application<'a> {
    pub fn new(
        ovr: &'a OVRController,
        config: &'a CleKeyConfig,
        app_status: Rc<dyn ApplicationStatus>,
        surfaces: Surfaces,
    ) -> Self {
        use input_method::*;
        let mut result = Self {
            ovr_controller: ovr,
            sign_input: SIGNS_TABLE,
            methods: VecDeque::from([JAPANESE_INPUT, ENGLISH_TABLE]),
            is_sign: false,
            kbd_status: KeyboardStatus {
                left: HandInfo::new(),
                right: HandInfo::new(),
                method: CleKeyInputTable {
                    starts_ime: false,
                    table: [CleKeyButton::empty(); 8 * 8],
                },
                button_idx: 0,
                buffer: String::new(),
                closing: false,
                candidates: vec![],
                candidates_idx: 0,
                henkan_using: None,
            },
            click_started: Instant::now(),
            app_status,
            config,
            surfaces,
        };

        result.set_plane(result.methods.front().unwrap());

        result
    }

    pub(crate) fn set_default_renderers(&mut self) {
        match self.config.ui_mode {
            UIMode::TwoRing => {
                self.surfaces.left_ring.renderer = renderer_fn::left_ring_renderer;
                self.surfaces.right_ring.renderer = renderer_fn::right_ring_renderer;
                self.surfaces.center_field.renderer = renderer_fn::center_field_renderer;
            }
            UIMode::OneRing => {
                self.surfaces.left_ring.renderer = renderer_fn::one_ring_renderer;
                self.surfaces.right_ring.renderer = renderer_fn::nop_renderer;
                self.surfaces.center_field.renderer = renderer_fn::center_field_renderer;
            }
        }
    }

    pub(crate) fn set_henkan_renderers(&mut self) {
        match self.config.ui_mode {
            UIMode::TwoRing => {
                self.surfaces.left_ring.renderer = renderer_fn::left_ring_henkan_renderer;
                self.surfaces.right_ring.renderer = renderer_fn::right_ring_henkan_renderer;
            }
            UIMode::OneRing => {
                self.surfaces.left_ring.renderer = renderer_fn::one_ring_henkan_renderer;
                self.surfaces.right_ring.renderer = renderer_fn::nop_renderer;
            }
        }
    }
}

struct Surfaces {
    left_ring: SurfaceInfo,
    right_ring: SurfaceInfo,
    center_field: SurfaceInfo,
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
            app.app_status = Rc::new(Inputting);
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
        app.ovr_controller.update_status(&mut app.kbd_status);

        match app.config.ui_mode {
            UIMode::TwoRing => {
                app.ovr_controller.show_overlay(OverlayPlane::Left);
                app.ovr_controller.show_overlay(OverlayPlane::Right);
            }
            UIMode::OneRing => {
                app.ovr_controller.show_overlay(OverlayPlane::Left);
            }
        }
        if !app.kbd_status.buffer.is_empty() {
            app.ovr_controller.show_overlay(OverlayPlane::Center);
        } else {
            app.ovr_controller.hide_overlay(OverlayPlane::Center);
        }

        if app.kbd_tick() {
            app.app_status = Rc::new(Waiting);
        }

        if app.ovr_controller.button_status(ButtonKind::SuspendInput) {
            app.app_status = Rc::new(Suspending)
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
            app.app_status = Rc::new(Inputting)
        }
    }
}

struct SurfaceInfo {
    gl_tex_id: GLuint,
    gl_framebuffer_id: GLuint,
    width: i32,
    height: i32,
    renderer: fn(context: &mut GraphicsContext, app: &Application) -> (),
}

impl SurfaceInfo {
    fn render(&self, context: &mut GraphicsContext, app: &Application) -> GLuint {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.gl_framebuffer_id);
            gl::Viewport(0, 0, self.width, self.height);
        }
        (self.renderer)(context, app);
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        self.gl_tex_id
    }
}

mod renderer_fn {
    use super::*;
    use crate::config;
    use crate::graphics::{draw_center, draw_ring};

    pub(crate) fn nop_renderer(_: &mut GraphicsContext, _: &Application) {}

    pub(crate) fn one_ring_renderer(context: &mut GraphicsContext, app: &Application) {
        draw_ring::<true>(
            context,
            &app.config.one_ring.ring,
            app.kbd_status.button_idx,
            app.kbd_status.left.selection,
            app.kbd_status.right.selection,
            app.kbd_status.left.stick,
            |current, opposite| app.kbd_status.method.table[8 * current + opposite],
        );
    }

    pub(crate) fn left_ring_renderer(context: &mut GraphicsContext, app: &Application) {
        draw_ring::<true>(
            context,
            &app.config.two_ring.left_ring,
            app.kbd_status.button_idx,
            app.kbd_status.left.selection,
            app.kbd_status.right.selection,
            app.kbd_status.left.stick,
            |current, opposite| app.kbd_status.method.table[8 * current + opposite],
        );
    }

    pub(crate) fn right_ring_renderer(context: &mut GraphicsContext, app: &Application) {
        draw_ring::<false>(
            context,
            &app.config.two_ring.right_ring,
            app.kbd_status.button_idx,
            app.kbd_status.right.selection,
            app.kbd_status.left.selection,
            app.kbd_status.right.stick,
            |current, opposite| app.kbd_status.method.table[current + 8 * opposite],
        );
    }

    pub(crate) fn center_field_renderer(context: &mut GraphicsContext, app: &Application) {
        draw_center(&app.kbd_status, &app.config.two_ring.completion, context);
    }

    pub(crate) fn henkan_renderer_impl(
        context: &mut GraphicsContext,
        config0: &CleKeyConfig,
        config: &config::RingOverlayConfig,
        hand: &HandInfo,
    ) {
        if config0.always_enter_paste {
            draw_ring::<false>(
                context,
                config,
                0,
                hand.selection,
                1,
                hand.stick,
                |current, _| ime_specific::BUTTONS[current],
            );
        } else {
            draw_ring::<false>(
                context,
                config,
                0,
                hand.selection,
                1,
                hand.stick,
                |current, _| ime_specific::BUTTONS_PASTE_OPTIONAL[current],
            );
        }
    }

    pub(crate) fn left_ring_henkan_renderer(context: &mut GraphicsContext, app: &Application) {
        henkan_renderer_impl(
            context,
            app.config,
            &app.config.two_ring.left_ring,
            &app.kbd_status.left,
        );
    }

    pub(crate) fn right_ring_henkan_renderer(context: &mut GraphicsContext, app: &Application) {
        henkan_renderer_impl(
            context,
            app.config,
            &app.config.two_ring.right_ring,
            &app.kbd_status.right,
        );
    }

    pub(crate) fn one_ring_henkan_renderer(context: &mut GraphicsContext, app: &Application) {
        match app.kbd_status.henkan_using {
            None => {
                henkan_renderer_impl(
                    context,
                    app.config,
                    &app.config.one_ring.ring,
                    &app.kbd_status.left,
                );
            }
            Some(LeftRight::Left) => {
                henkan_renderer_impl(
                    context,
                    app.config,
                    &app.config.one_ring.ring,
                    &app.kbd_status.left,
                );
            }
            Some(LeftRight::Right) => {
                henkan_renderer_impl(
                    context,
                    app.config,
                    &app.config.one_ring.ring,
                    &app.kbd_status.right,
                );
            }
        }
    }
}

fn create_surface(width: i32, height: i32) -> SurfaceInfo {
    let mut gl_tex_id = 0;
    let mut gl_framebuffer_id = 0;
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

        gl::GenFramebuffers(1, &mut gl_framebuffer_id);
        gl::BindFramebuffer(gl::FRAMEBUFFER, gl_framebuffer_id);

        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl_tex_id, 0);
        gl::DrawBuffers(1, [gl::COLOR_ATTACHMENT0].as_ptr());

        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!(
                "Framebuffer rendering failed: {:x}",
                gl::CheckFramebufferStatus(gl::FRAMEBUFFER)
            );
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    SurfaceInfo {
        gl_tex_id,
        gl_framebuffer_id,
        width,
        height,
        renderer: renderer_fn::nop_renderer,
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
    pub(crate) fn selection_changed(&self) -> bool {
        self.selection != self.selection_old
    }

    pub(crate) fn click_started(&self) -> bool {
        !self.clicking_old && self.clicking
    }
}

impl HandInfo {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            stick: Vec2::new(0.0, 0.0),
            selection: -1,
            selection_old: -1,
            clicking: false,
            clicking_old: false,
        }
    }
}

pub struct KeyboardStatus {
    left: HandInfo,
    right: HandInfo,
    method: CleKeyInputTable<'static>,
    button_idx: usize,
    buffer: String,
    closing: bool,
    candidates: Vec<HenkanCandidate>,
    candidates_idx: usize,
    henkan_using: Option<LeftRight>,
}

pub struct HenkanCandidate {
    candidates: Vec<String>,
    index: usize,
}

impl KeyboardStatus {
    pub(crate) fn is_selecting(&self) -> bool {
        self.left.selection != -1 && self.right.selection != -1
    }

    pub(crate) fn click_started(&self) -> bool {
        // prev: both not clicking
        // now: either clicking
        (!self.left.clicking_old && !self.right.clicking_old)
            && (self.left.clicking || self.right.clicking)
    }

    pub(crate) fn click_stopped(&self) -> bool {
        // prev: either clicking
        // now: both not clicking
        (self.left.clicking_old || self.right.clicking_old)
            && (!self.left.clicking && !self.right.clicking)
    }

    pub(crate) fn selection_changed(&self) -> bool {
        self.left.selection_changed() || self.right.selection_changed()
    }

    pub(crate) fn clicking(&self) -> bool {
        self.left.clicking || self.right.clicking
    }

    pub(crate) fn selecting_button(&self) -> Option<CleKeyButton<'static>> {
        if self.is_selecting() {
            Some(self.method.table[(self.left.selection * 8 + self.right.selection) as usize])
        } else {
            None
        }
    }
}

impl<'a> Application<'a> {
    pub(crate) fn kbd_tick(&mut self) -> bool {
        if self.kbd_status.candidates.is_empty() {
            self.kbd_inputting_tick()
        } else {
            self.kbd_henkan_tick()
        }
    }

    pub(crate) fn kbd_inputting_tick(&mut self) -> bool {
        if let Some(button) = self.kbd_status.selecting_button() {
            if self.kbd_status.click_started() || self.kbd_status.selection_changed() {
                self.click_started = Instant::now();
                self.kbd_status.button_idx = 0
            } else if self.kbd_status.clicking() {
                if !button.0.is_empty() {
                    let dur = Instant::now().duration_since(self.click_started);
                    let millis = dur.as_millis();
                    self.kbd_status.button_idx =
                        (((millis + self.config.click.offset) / self.config.click.length)
                            % button.0.len() as u128) as usize;
                } else {
                    self.kbd_status.button_idx = 0;
                }
            } else if self.kbd_status.click_stopped() {
                info!(
                    "clicked: {}ms",
                    Instant::now()
                        .duration_since(self.click_started)
                        .as_millis()
                );
                if let Some(action) = button.0.get(self.kbd_status.button_idx).map(|x| &x.action) {
                    self.do_input_action(action)
                }
                self.kbd_status.button_idx = 0;
                if take(&mut self.kbd_status.closing) {
                    return true;
                }
            }
        }

        for x in HardKeyButton::VALUES {
            if self.ovr_controller.click_started(x) {
                match x {
                    HardKeyButton::CloseButton => return true,
                    #[allow(unreachable_patterns)]
                    _ => {
                        todo!()
                        //let action = self.status.method.on_hard_input(x);
                        //self.do_input_action(action)
                    }
                }
            }
        }
        false
    }

    pub(crate) fn kbd_henkan_tick(&mut self) -> bool {
        fn get_input_action(
            config: &CleKeyConfig,
            hand: &HandInfo,
        ) -> Option<&'static InputNextAction> {
            if hand.selection != -1 && hand.click_started() {
                let buttons = if config.always_enter_paste {
                    &ime_specific::BUTTONS
                } else {
                    &ime_specific::BUTTONS_PASTE_OPTIONAL
                };
                if let Some(action) = buttons[hand.selection as usize]
                    .0
                    .first()
                    .map(|x| &x.action)
                {
                    return Some(action);
                }
            }
            None
        }
        fn action_left(app: &mut Application) {
            if let Some(action) = get_input_action(app.config, &app.kbd_status.left) {
                app.do_input_action(action);
            }
        }
        fn action_right(app: &mut Application) {
            if let Some(action) = get_input_action(app.config, &app.kbd_status.right) {
                app.do_input_action(action);
            }
        }
        match self.config.ui_mode {
            UIMode::TwoRing => {
                action_left(self);
                action_right(self);
            }
            UIMode::OneRing => match self.kbd_status.henkan_using {
                None => {
                    if self.kbd_status.left.stick != Vec2::ZERO {
                        self.kbd_status.henkan_using = Some(LeftRight::Left);
                    } else if self.kbd_status.right.stick != Vec2::ZERO {
                        self.kbd_status.henkan_using = Some(LeftRight::Right);
                    }
                }
                Some(LeftRight::Left) => {
                    action_left(self);
                    if self.kbd_status.left.stick == Vec2::ZERO {
                        self.kbd_status.henkan_using = None
                    }
                }
                Some(LeftRight::Right) => {
                    action_right(self);
                    if self.kbd_status.right.stick == Vec2::ZERO {
                        self.kbd_status.henkan_using = None
                    }
                }
            },
        }

        for x in HardKeyButton::VALUES {
            #[allow(unreachable_patterns, clippy::single_match)]
            if self.ovr_controller.click_started(x) {
                match x {
                    HardKeyButton::CloseButton => return true,
                    // nop
                    _ => (),
                }
            }
        }

        false
    }

    fn do_input_action(&mut self, action: &InputNextAction) {
        match action {
            InputNextAction::EnterChar(c) => {
                if self.config.always_use_buffer
                    || self.kbd_status.method.starts_ime
                    || !self.kbd_status.buffer.is_empty()
                {
                    self.kbd_status.buffer.push(*c);
                    self.set_inputting_table();
                } else {
                    os::enter_char(*c)
                }
            }
            InputNextAction::Extra(f) => f(&mut self.kbd_status),
            InputNextAction::Intrinsic(f) => f(self),
        }
    }

    fn move_to_next_plane(&mut self) {
        self.is_sign = false;
        // rotate
        self.methods.rotate_left(1);
        // and clear
        self.set_plane(self.methods.front().unwrap());
    }

    fn swap_sign_plane(&mut self) {
        if self.is_sign {
            self.is_sign = false;
            self.set_plane(self.methods.front().unwrap());
        } else {
            self.is_sign = true;
            self.set_plane(self.sign_input);
        }
    }

    pub fn flush(&mut self, force_paste: bool) -> bool {
        let mut builder = String::new();
        let buffer = if self.kbd_status.candidates.is_empty() {
            self.kbd_status.buffer.as_str()
        } else {
            for x in &self.kbd_status.candidates {
                builder.push_str(&x.candidates[x.index]);
            }
            &builder
        };
        let mut success = true;
        if !buffer.is_empty() {
            let enter = force_paste || self.config.always_enter_paste;
            if enter {
                success = os::enter_text(buffer)
            } else {
                success = os::copy_text(buffer);
            }
        }
        if success {
            self.set_inputted_table();
            self.kbd_status.buffer.clear();
            self.kbd_status.candidates.clear();
        }
        success
    }

    fn close_key(mgr: &mut Application) {
        debug_assert!(mgr.kbd_status.buffer.is_empty());
        mgr.kbd_status.closing = true;
    }

    fn henkan_key(mgr: &mut Application) {
        debug_assert!(!mgr.kbd_status.buffer.is_empty());

        // query percent-encode set     = C0 control percent-encode set + " "#<>"
        // path percent-encode set      = query percent-encode set + "?^`{}"
        // userinfo percent-encode set  = path percent-encode set + "/:;=@[\]|"
        // component percent-encode set = userinfo percent-encode set + "$%&+,"
        const COMPONENT_ENCODE_SET: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
            // query percent-encode set
            .add(b' ')
            .add(b'"')
            .add(b'#')
            .add(b'<')
            .add(b'>')
            // path percent-encode set
            .add(b'?')
            .add(b'^')
            .add(b'`')
            .add(b'{')
            .add(b'}')
            // userinfo percent-encode set
            .add(b'/')
            .add(b':')
            .add(b';')
            .add(b'=')
            .add(b'@')
            .add(b'[')
            .add(b'\\')
            .add(b']')
            .add(b'|')
            // component percent-encode set
            .add(b'$')
            .add(b'%')
            .add(b'&')
            .add(b'+')
            .add(b',');

        if let Ok(mut response) = reqwest::blocking::get(format!(
            "https://www.google.com/transliterate?langpair=ja-Hira|ja&text={text}",
            text =
                percent_encoding::utf8_percent_encode(&mgr.kbd_status.buffer, COMPONENT_ENCODE_SET)
        ))
        .and_then(|x| x.json::<Vec<(String, Vec<String>)>>())
        {
            if response.is_empty() {
                response = vec![(
                    mgr.kbd_status.buffer.clone(),
                    vec![mgr.kbd_status.buffer.clone()],
                )];
            }
            mgr.kbd_status.candidates_idx = 0;
            mgr.kbd_status.henkan_using = None;
            mgr.kbd_status.candidates = response
                .into_iter()
                .map(|(original, candidates)| HenkanCandidate {
                    candidates: Self::add_candidates(original, candidates),
                    index: 0,
                })
                .collect();
            mgr.set_henkan_renderers();
        };
    }

    fn add_candidates(original: String, mut vec: Vec<String>) -> Vec<String> {
        if !vec.contains(&original) {
            vec.push(original.clone())
        }

        // TODO: add カタカナ option?

        vec
    }

    fn new_line_key(mgr: &mut Application) {
        debug_assert!(mgr.kbd_status.buffer.is_empty());
        os::enter_enter();
    }

    fn backspace_key(mgr: &mut Application) {
        if mgr.kbd_status.buffer.pop().is_some() {
            if mgr.kbd_status.buffer.is_empty() {
                mgr.set_inputted_table();
            }
        } else {
            os::enter_backspace();
        }
    }

    fn space_key(mgr: &mut Application) {
        if mgr.kbd_status.buffer.is_empty() {
            os::enter_char(' ');
        } else {
            mgr.kbd_status.buffer.push(' ');
        }
    }

    fn next_plane_key(mgr: &mut Application) {
        mgr.move_to_next_plane()
    }

    fn sign_plane_key(mgr: &mut Application) {
        mgr.swap_sign_plane()
    }
}

macro_rules! builtin_button {
    ($char: literal = $func: expr) => {
        CleKeyButton(&[CleKeyButtonAction {
            shows: $char,
            action: InputNextAction::Intrinsic($func),
        }])
    };
}

impl<'ovr> Application<'ovr> {
    fn set_plane(&mut self, table: &CleKeyInputTable<'static>) {
        use input_method::*;
        self.kbd_status.method.clone_from(table);

        use Application as App;
        self.kbd_status.method.table[6 * 8 + 6] = builtin_button!("⌫" = App::backspace_key);
        self.kbd_status.method.table[6 * 8 + 7] = builtin_button!("␣" = App::space_key);

        // 🌐
        self.kbd_status.method.table[7 * 8 + 6] =
            builtin_button!("\u{1F310}" = App::next_plane_key);
        self.kbd_status.method.table[7 * 8 + 7] = builtin_button!("#+=" = App::sign_plane_key);

        if self.kbd_status.buffer.is_empty() {
            self.set_inputted_table();
        } else {
            self.set_inputting_table();
        }
    }

    fn set_inputted_table(&mut self) {
        use input_method::*;
        self.kbd_status.method.table[5 * 8 + 6] = builtin_button!("Close" = Application::close_key);
        self.kbd_status.method.table[5 * 8 + 7] = builtin_button!("⏎" = Application::new_line_key);
    }

    fn set_inputting_table(&mut self) {
        use input_method::*;
        self.kbd_status.method.table[5 * 8 + 6] = builtin_button!("変換" = Application::henkan_key);
        self.kbd_status.method.table[5 * 8 + 7] = CleKeyButton::empty();
    }
}

mod ime_specific {
    use crate::Application;
    use crate::input_method::{CleKeyButton, CleKeyButtonAction, InputNextAction};

    pub(crate) static BUTTONS: [CleKeyButton; 8] = [
        builtin_button!("↑" = up_key),
        builtin_button!("Cancel" = cancel_key),
        builtin_button!("→" = right_key),
        CleKeyButton::empty(),
        builtin_button!("↓" = down_key),
        CleKeyButton::empty(),
        builtin_button!("←" = left_key),
        builtin_button!("確定" = kakutei_key),
    ];

    pub(crate) static BUTTONS_PASTE_OPTIONAL: [CleKeyButton; 8] = [
        builtin_button!("↑" = up_key),
        builtin_button!("Cancel" = cancel_key),
        builtin_button!("→" = right_key),
        CleKeyButton::empty(),
        builtin_button!("↓" = down_key),
        builtin_button!("Copy" = kakutei_key),
        builtin_button!("←" = left_key),
        builtin_button!("入力" = kakutei_paste_key),
    ];

    fn cancel_key(mgr: &mut Application) {
        mgr.kbd_status.candidates.clear();
        mgr.kbd_status.candidates_idx = 0;
        mgr.set_default_renderers();
    }

    fn kakutei_key(mgr: &mut Application) {
        debug_assert!(!mgr.kbd_status.buffer.is_empty());
        if mgr.flush(false) {
            mgr.set_default_renderers();
        }
    }

    fn kakutei_paste_key(mgr: &mut Application) {
        if mgr.flush(true) {
            mgr.set_default_renderers();
        }
    }

    fn up_key(mgr: &mut Application) {
        let candidate = &mut mgr.kbd_status.candidates[mgr.kbd_status.candidates_idx];
        if candidate.index == 0 {
            candidate.index = candidate.candidates.len() - 1;
        } else {
            candidate.index -= 1;
        }
    }

    fn down_key(mgr: &mut Application) {
        let candidate = &mut mgr.kbd_status.candidates[mgr.kbd_status.candidates_idx];
        candidate.index += 1;
        if candidate.index == candidate.candidates.len() {
            candidate.index = 0;
        }
    }

    fn left_key(mgr: &mut Application) {
        if mgr.kbd_status.candidates_idx == 0 {
            mgr.kbd_status.candidates_idx = mgr.kbd_status.candidates.len() - 1;
        } else {
            mgr.kbd_status.candidates_idx -= 1;
        }
    }

    fn right_key(mgr: &mut Application) {
        mgr.kbd_status.candidates_idx += 1;
        if mgr.kbd_status.candidates_idx == mgr.kbd_status.candidates.len() {
            mgr.kbd_status.candidates_idx = 0;
        }
    }
}
