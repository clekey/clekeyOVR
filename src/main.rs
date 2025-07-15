#[macro_use]
mod utils;
mod config;
#[cfg(feature = "debug_window")]
mod debug_graphics;
mod global;
mod graphics;
mod input_method;
mod licenses;
mod os;
mod ovr_controller;
mod resources;

use crate::config::{load_config, CleKeyConfig, UIMode};
use crate::input_method::{CleKeyButton, CleKeyInputTable, HardKeyButton, InputNextAction};
use crate::ovr_controller::{ActionSetKind, ButtonKind, OVRController, OverlayPlane};
use gl::types::GLuint;
use glam::Vec2;
use glfw::{Context, OpenGlProfileHint, WindowHint};
use graphics::FontInfo;
use log::info;
use skia_safe::font_style::{Slant, Weight, Width};
use skia_safe::gpu::gl::TextureInfo;
use skia_safe::gpu::{BackendTexture, Mipmapped, SurfaceOrigin};
use skia_safe::textlayout::FontCollection;
use skia_safe::{gpu, ColorType, FontMgr, FontStyle, Surface};
use std::collections::VecDeque;
use std::mem::take;
use std::ptr::null;
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};

const WINDOW_HEIGHT: i32 = 1024;
const WINDOW_WIDTH: i32 = 1024;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum LeftRight {
    Left = 0,
    Right = 1,
}

#[cfg(feature = "debug_window")]
compile_error!("debug_window feature is not supported");

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
    // glfw initialization
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(WindowHint::DoubleBuffer(true));
    glfw.window_hint(WindowHint::ContextVersionMajor(4));
    glfw.window_hint(WindowHint::ContextVersionMinor(1));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::Resizable(false));
    glfw.window_hint(WindowHint::CocoaRetinaFramebuffer(false));
    glfw.window_hint(WindowHint::Visible(cfg!(feature = "debug_window")));

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

    #[cfg(windows)]
    os::set_hwnd(window.get_win32_window());

    // gl crate initialization
    let loader = |s: &str| glfw.get_proc_address_raw(s);
    gl::load_with(loader);

    let skia_interfaces = gpu::gl::Interface::new_load_with(loader).expect("initializing skia api");
    let mut skia_ctx =
        gpu::direct_contexts::make_gl(skia_interfaces, None).expect("skia gpu context creation");

    // debug block
    #[cfg(feature = "debug_window")]
    let debug_renderer = unsafe {
        window.make_current();
        debug_graphics::DebugRenderer::init()
    };

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
            left_ring: create_surface(&mut skia_ctx, WINDOW_WIDTH, WINDOW_HEIGHT),
            right_ring: create_surface(&mut skia_ctx, WINDOW_WIDTH, WINDOW_HEIGHT),
            center_field: create_surface(&mut skia_ctx, WINDOW_WIDTH, WINDOW_HEIGHT / 2),
        },
    );

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
            info!("loaded: {:?}", face);
        }
    }
    info!("font_families: {:?}", font_families);

    // TODO: find way to use Noto Sans in rendering instead of system fonts
    fonts.set_default_font_manager(Some(font_mgr), None);
    info!(
        "find_typefaces: {:?}",
        fonts.find_typefaces(
            &font_families,
            FontStyle::new(Weight::MEDIUM, Width::NORMAL, Slant::Upright)
        )
    );

    let fonts = FontInfo {
        collection: fonts,
        families: &font_families,
    };

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

    while !window.should_close() {
        let frame_end_expected = Instant::now() + frame_duration;
        glfw.poll_events();
        for (_, _event) in glfw::flush_messages(&events) {
            #[cfg(feature = "debug_control")]
            ovr_controller.accept_debug_control(_event);
        }

        // TODO: openvr tick

        app.app_status.clone().tick(&mut app);

        // Surface::flush() does not work as expect but the following is working.
        //  Surface::image_snapshot(<surface>).backend_texture(true);
        // I don't know why this is working but It's working so I'm using that.

        ovr_controller.draw_if_visible(LeftRight::Left.into(), || {
            let surface = &app.surfaces.left_ring;
            (surface.renderer)(surface.surface.clone(), &app, &fonts);
            gpu::images::get_backend_texture_from_image(
                &app.surfaces.left_ring.surface.image_snapshot(),
                true,
            );
            app.surfaces.left_ring.gl_tex_id
        });

        ovr_controller.draw_if_visible(LeftRight::Right.into(), || {
            let surface = &app.surfaces.right_ring;
            (surface.renderer)(surface.surface.clone(), &app, &fonts);
            gpu::images::get_backend_texture_from_image(
                &app.surfaces.right_ring.surface.image_snapshot(),
                true,
            );
            app.surfaces.right_ring.gl_tex_id
        });

        ovr_controller.draw_if_visible(OverlayPlane::Center, || {
            let surface = &app.surfaces.center_field;
            (surface.renderer)(surface.surface.clone(), &app, &fonts);
            gpu::images::get_backend_texture_from_image(
                &app.surfaces.center_field.surface.image_snapshot(),
                true,
            );
            app.surfaces.center_field.gl_tex_id
        });

        #[cfg(feature = "debug_window")]
        {
            window.make_current();
            unsafe {
                // wipe the drawing surface clear
                let framebuffer = gl_get_uint::<1>(gl::FRAMEBUFFER_BINDING)[0];
                let viewport = gl_get_int::<4>(gl::VIEWPORT);
                let color_clear = gl_get_float::<4>(gl::COLOR_CLEAR_VALUE);
                let texture_binding_2d = gl_get_uint::<1>(gl::TEXTURE_BINDING_2D)[0];
                let active_texture = gl_get_uint::<1>(gl::ACTIVE_TEXTURE)[0];
                if false {
                    info!("================================");
                    info!("FRAMEBUFFER_BINDING: {framebuffer:?}");
                    info!("VIEWPORT: {viewport:?}");
                    info!("COLOR_CLEAR_VALUE: {color_clear:?}");
                    info!("TEXTURE_BINDING_2D: {texture_binding_2d:?}");
                    info!("ACTIVE_TEXTURE: {active_texture:?}");
                    info!("================================");
                }

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

                window.swap_buffers();

                gl::ActiveTexture(active_texture);
                gl::BindTexture(gl::TEXTURE_2D, texture_binding_2d);
                gl::ClearColor(
                    color_clear[0],
                    color_clear[1],
                    color_clear[2],
                    color_clear[3],
                );
                gl::Viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
                gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            }
        }
        let (average, one_frame) = fps_calc.on_frame();
        print!("FPS: {average:7.3} fps ({one_frame:7.3} fps in short period)\r");
        // sleep for next frame.
        sleep(frame_end_expected.duration_since(Instant::now()));
    }
}

#[cfg(feature = "debug_window")]
unsafe fn gl_get_uint<const N: usize>(name: gl::types::GLenum) -> [GLuint; N] {
    gl_get_int::<N>(name).map(|x| x as GLuint)
}

#[cfg(feature = "debug_window")]
unsafe fn gl_get_int<const N: usize>(name: gl::types::GLenum) -> [gl::types::GLint; N] {
    let mut value = [0; N];
    gl::GetIntegerv(name, value.as_mut_ptr());
    value
}

#[cfg(feature = "debug_window")]
unsafe fn gl_get_float<const N: usize>(name: gl::types::GLenum) -> [gl::types::GLfloat; N] {
    let mut value = [0.0; N];
    gl::GetFloatv(name, value.as_mut_ptr());
    value
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
    surface: Surface,
    renderer: fn(Surface, app: &Application, fonts: &FontInfo) -> (),
}

mod renderer_fn {
    use super::*;
    use crate::config;
    use crate::graphics::{draw_center, draw_ring};

    pub(crate) fn nop_renderer(_: Surface, _: &Application, _: &FontInfo) {}

    pub(crate) fn one_ring_renderer(mut surface: Surface, app: &Application, fonts: &FontInfo) {
        draw_ring::<true>(
            &mut surface,
            &app.config.one_ring.ring,
            fonts,
            app.kbd_status.button_idx,
            app.kbd_status.left.selection,
            app.kbd_status.right.selection,
            app.kbd_status.left.stick,
            |current, opposite| app.kbd_status.method.table[8 * current + opposite],
        );
    }

    pub(crate) fn left_ring_renderer(mut surface: Surface, app: &Application, fonts: &FontInfo) {
        draw_ring::<true>(
            &mut surface,
            &app.config.two_ring.left_ring,
            fonts,
            app.kbd_status.button_idx,
            app.kbd_status.left.selection,
            app.kbd_status.right.selection,
            app.kbd_status.left.stick,
            |current, opposite| app.kbd_status.method.table[8 * current + opposite],
        );
    }

    pub(crate) fn right_ring_renderer(mut surface: Surface, app: &Application, fonts: &FontInfo) {
        draw_ring::<false>(
            &mut surface,
            &app.config.two_ring.right_ring,
            fonts,
            app.kbd_status.button_idx,
            app.kbd_status.right.selection,
            app.kbd_status.left.selection,
            app.kbd_status.right.stick,
            |current, opposite| app.kbd_status.method.table[current + 8 * opposite],
        );
    }

    pub(crate) fn center_field_renderer(mut surface: Surface, app: &Application, fonts: &FontInfo) {
        draw_center(
            &app.kbd_status,
            &app.config.two_ring.completion,
            fonts,
            &mut surface,
        );
    }

    pub(crate) fn henkan_renderer_impl(
        mut surface: Surface,
        config0: &CleKeyConfig,
        config: &config::RingOverlayConfig,
        hand: &HandInfo,
        fonts: &FontInfo,
    ) {
        if config0.always_enter_paste {
            draw_ring::<false>(
                &mut surface,
                config,
                fonts,
                0,
                hand.selection,
                1,
                hand.stick,
                |current, _| ime_specific::BUTTONS[current],
            );
        } else {
            draw_ring::<false>(
                &mut surface,
                config,
                fonts,
                0,
                hand.selection,
                1,
                hand.stick,
                |current, _| ime_specific::BUTTONS_PASTE_OPTIONAL[current],
            );
        }
    }

    pub(crate) fn left_ring_henkan_renderer(surface: Surface, app: &Application, fonts: &FontInfo) {
        henkan_renderer_impl(
            surface,
            app.config,
            &app.config.two_ring.left_ring,
            &app.kbd_status.left,
            fonts,
        );
    }

    pub(crate) fn right_ring_henkan_renderer(
        surface: Surface,
        app: &Application,
        fonts: &FontInfo,
    ) {
        henkan_renderer_impl(
            surface,
            app.config,
            &app.config.two_ring.right_ring,
            &app.kbd_status.right,
            fonts,
        );
    }

    pub(crate) fn one_ring_henkan_renderer(surface: Surface, app: &Application, fonts: &FontInfo) {
        match app.kbd_status.henkan_using {
            None => {
                henkan_renderer_impl(
                    surface,
                    app.config,
                    &app.config.one_ring.ring,
                    &app.kbd_status.left,
                    fonts,
                );
            }
            Some(LeftRight::Left) => {
                henkan_renderer_impl(
                    surface,
                    app.config,
                    &app.config.one_ring.ring,
                    &app.kbd_status.left,
                    fonts,
                );
            }
            Some(LeftRight::Right) => {
                henkan_renderer_impl(
                    surface,
                    app.config,
                    &app.config.one_ring.ring,
                    &app.kbd_status.right,
                    fonts,
                );
            }
        }
    }
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
                ..TextureInfo::default()
            },
        )
    };
    let surface = gpu::surfaces::wrap_backend_texture(
        context,
        &backend_texture,
        SurfaceOrigin::BottomLeft,
        None,
        ColorType::RGBA8888,
        None,
        None,
    )
    .expect("creating surface");

    SurfaceInfo {
        gl_tex_id,
        surface,
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
                if button.0.len() != 0 {
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
        return false;
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
                if let Some(action) = buttons[hand.selection as usize].0.get(0).map(|x| &x.action) {
                    return Some(action);
                }
            }
            None
        }
        fn action_left(app: &mut Application) {
            if let Some(action) = get_input_action(&app.config, &app.kbd_status.left) {
                app.do_input_action(action);
            }
        }
        fn action_right(app: &mut Application) {
            if let Some(action) = get_input_action(&app.config, &app.kbd_status.right) {
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
            if self.ovr_controller.click_started(x) {
                match x {
                    HardKeyButton::CloseButton => return true,
                    // nop
                    #[allow(unreachable_patterns)]
                    _ => (),
                }
            }
        }

        return false;
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
            &self.kbd_status.buffer
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
                success = os::enter_text(&buffer)
            } else {
                success = os::copy_text(&buffer);
            }
        }
        if success {
            self.set_inputted_table();
            self.kbd_status.buffer.clear();
            self.kbd_status.candidates.clear();
        }
        return success;
    }

    fn close_key(mgr: &mut Application) {
        debug_assert!(mgr.kbd_status.buffer.is_empty());
        mgr.kbd_status.closing = true;
    }

    fn henkan_key(mgr: &mut Application) {
        debug_assert!(!mgr.kbd_status.buffer.is_empty());

        const QUERY: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
            .add(b' ')
            .add(b'"')
            .add(b'#')
            .add(b'<')
            .add(b'+')
            .add(b'>');

        if let Some(response) = reqwest::blocking::get(format!(
            "https://www.google.com/transliterate?langpair=ja-Hira|ja&text={text}",
            text = percent_encoding::utf8_percent_encode(&mgr.kbd_status.buffer, QUERY)
        ))
        .and_then(|x| x.json::<Vec<(String, Vec<String>)>>())
        .ok()
        {
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
        if let Some(_) = mgr.kbd_status.buffer.pop() {
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
    use crate::input_method::{CleKeyButton, CleKeyButtonAction, InputNextAction};
    use crate::Application;

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
        builtin_button!("確定" = kakutei_key),
        builtin_button!("←" = left_key),
        builtin_button!("貼付" = kakutei_paste_key),
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
