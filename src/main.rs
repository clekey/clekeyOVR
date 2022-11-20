#[macro_use]
mod utils;
mod config;
mod global;
mod graphics;
mod input_method;
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
use skia_safe::{gpu, AlphaType, ColorType, FontMgr, FontStyle, Image, Surface};
#[cfg(feature = "debug_window")]
use skia_safe::{gpu::BackendRenderTarget, Rect, SamplingOptions};
use std::collections::VecDeque;
use std::ptr::null;
use std::rc::Rc;
use std::time::Instant;

const WINDOW_HEIGHT: i32 = 1024;
const WINDOW_WIDTH: i32 = 1024;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum LeftRight {
    Left = 0,
    Right = 1,
}

fn main() {
    simple_logger::init().unwrap();
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
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
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
        config: &config,
        surfaces: Surfaces {
            left_ring: create_surface(&mut skia_ctx.clone().into(), WINDOW_WIDTH, WINDOW_HEIGHT),
            right_ring: create_surface(&mut skia_ctx.clone().into(), WINDOW_WIDTH, WINDOW_HEIGHT),
            center_field: create_surface(
                &mut skia_ctx.clone().into(),
                WINDOW_WIDTH,
                WINDOW_HEIGHT / 2,
            ),
        },
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

    let fonts = FontInfo {
        collection: fonts,
        families: &font_families,
    };

    // gl initialiation

    app.set_default_renderers();

    //frame.clear_color();

    while !window.should_close() {
        glfw.poll_events();
        for (_, _event) in glfw::flush_messages(&events) {
            #[cfg(feature = "debug_control")]
            ovr_controller.accept_debug_control(_event);
        }

        // TODO: openvr tick

        app.status.clone().tick(&mut app);

        ovr_controller.draw_if_visible(LeftRight::Left.into(), || {
            let surface = &app.surfaces.left_ring;
            (surface.renderer)(surface.surface.clone(), &app, &fonts);
            app.surfaces.left_ring.surface.flush();
            app.surfaces.left_ring.gl_tex_id
        });

        ovr_controller.draw_if_visible(LeftRight::Right.into(), || {
            let surface = &app.surfaces.right_ring;
            (surface.renderer)(surface.surface.clone(), &app, &fonts);
            app.surfaces.right_ring.surface.flush();
            app.surfaces.right_ring.gl_tex_id
        });

        ovr_controller.draw_if_visible(OverlayPlane::Center, || {
            let surface = &app.surfaces.center_field;
            (surface.renderer)(surface.surface.clone(), &app, &fonts);
            app.surfaces.center_field.surface.flush();
            app.surfaces.center_field.gl_tex_id
        });

        #[cfg(feature = "debug_window")]
        {
            let canvas = window_surface.canvas();
            let width = WINDOW_WIDTH as f32;
            let half_width = width / 2.0;
            canvas
                .draw_image_rect_with_sampling_options(
                    &app.surfaces.left_ring.image,
                    None,
                    Rect::from_xywh(0.0, 0.0, half_width, half_width),
                    SamplingOptions::default(),
                    &Default::default(),
                )
                .draw_image_rect_with_sampling_options(
                    &app.surfaces.right_ring.image,
                    None,
                    Rect::from_xywh(half_width, 0.0, half_width, half_width),
                    SamplingOptions::default(),
                    &Default::default(),
                )
                .draw_image_rect_with_sampling_options(
                    &app.surfaces.center_field.image,
                    None,
                    Rect::from_xywh(0.0, half_width, width, half_width),
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
    config: &'a CleKeyConfig,
    surfaces: Surfaces,
}

impl<'a> Application<'a> {
    pub(crate) fn set_default_renderers(&mut self) {
        match self.config.ui_mode {
            UIMode::TwoRing => {
                self.surfaces.left_ring.renderer = renderer_fn::left_ring_renderer;
                self.surfaces.right_ring.renderer = renderer_fn::right_ring_renderer;
                self.surfaces.center_field.renderer = renderer_fn::center_field_renderer;
            }
            UIMode::OneRing => {
                self.surfaces.left_ring.renderer = renderer_fn::left_ring_renderer;
                self.surfaces.right_ring.renderer = renderer_fn::nop_renderer;
                self.surfaces.center_field.renderer = renderer_fn::center_field_renderer;
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

        match app.config.ui_mode {
            UIMode::TwoRing => {
                app.ovr_controller.show_overlay(OverlayPlane::Left);
                app.ovr_controller.show_overlay(OverlayPlane::Right);
            }
            UIMode::OneRing => {
                app.ovr_controller.show_overlay(OverlayPlane::Left);
            }
        }
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
    renderer: fn(Surface, app: &Application, fonts: &FontInfo) -> (),
}

mod renderer_fn {
    use super::*;
    use crate::graphics::{draw_center, draw_ring};

    pub(crate) fn nop_renderer(_: Surface, _: &Application, _: &FontInfo) {}

    pub(crate) fn left_ring_renderer(mut surface: Surface, app: &Application, fonts: &FontInfo) {
        draw_ring::<true, true>(
            &app.keyboard.status,
            &app.config.two_ring.left_ring,
            fonts,
            &mut surface,
        );
    }

    pub(crate) fn right_ring_renderer(mut surface: Surface, app: &Application, fonts: &FontInfo) {
        draw_ring::<false, false>(
            &app.keyboard.status,
            &app.config.two_ring.left_ring,
            fonts,
            &mut surface,
        );
    }

    pub(crate) fn center_field_renderer(mut surface: Surface, app: &Application, fonts: &FontInfo) {
        draw_center(
            &app.keyboard.status,
            &app.config.two_ring.completion,
            fonts,
            &mut surface,
        );
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
    sign_input: &'static CleKeyInputTable<'static>,
    methods: VecDeque<&'static CleKeyInputTable<'static>>,
    is_sign: bool,
    status: KeyboardStatus,
    click_started: Instant,
    config: &'ovr CleKeyConfig,
}

impl<'ovr> KeyboardManager<'ovr> {
    pub fn new(ovr: &'ovr OVRController, config: &'ovr CleKeyConfig) -> Self {
        use input_method::*;
        let mut result = Self {
            ovr_controller: ovr,
            sign_input: SIGNS_TABLE,
            methods: VecDeque::from([JAPANESE_INPUT, ENGLISH_TABLE]),
            is_sign: false,
            status: KeyboardStatus {
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
            },
            click_started: Instant::now(),
            config,
        };

        result.set_plane(result.methods.front().unwrap());

        result
    }

    pub(crate) fn tick(&mut self) -> bool {
        if let Some(button) = self.status.selecting_button() {
            if self.status.click_started() || self.status.selection_changed() {
                self.click_started = Instant::now();
                self.status.button_idx = 0
            } else if self.status.clicking() {
                if button.0.len() != 0 {
                    let dur = Instant::now().duration_since(self.click_started);
                    let millis = dur.as_millis();
                    println!("since: {}, {:?}", millis, self.click_started);
                    self.status.button_idx =
                        (((millis + self.config.click.offset) / self.config.click.length)
                            % button.0.len() as u128) as usize;
                } else {
                    self.status.button_idx = 0;
                }
            } else if self.status.click_stopped() {
                if let Some(action) = button.0.get(self.status.button_idx).map(|x| &x.action) {
                    self.do_input_action(action)
                }
                self.status.button_idx = 0;
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

    fn do_input_action(&mut self, action: &InputNextAction) {
        match action {
            InputNextAction::EnterChar(c) => {
                if self.status.method.starts_ime || !self.status.buffer.is_empty() {
                    self.status.buffer.push(*c);
                    self.set_inputting_table();
                } else {
                    os::enter_char(*c)
                }
            }
            InputNextAction::Extra(f) => f(&mut self.status),
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

    pub fn flush(&mut self) {
        let buffer = if self.status.candidates.is_empty() {
            std::mem::take(&mut self.status.buffer)
        } else {
            let mut builder = String::new();
            for x in &self.status.candidates {
                builder.push_str(&x.candidates[x.index]);
            }
            self.status.buffer.clear();
            self.status.candidates.clear();
            builder
        };
        self.set_inputted_table();
        if !buffer.is_empty() {
            os::copy_text_and_enter_paste_shortcut(&buffer);
        }
    }

    fn close_key(mgr: &mut KeyboardManager) {
        debug_assert!(mgr.status.buffer.is_empty());
        mgr.status.closing = true;
    }

    fn henkan_key(mgr: &mut KeyboardManager) {
        debug_assert!(!mgr.status.buffer.is_empty());
        // nop currently

        const QUERY: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
            .add(b' ')
            .add(b'"')
            .add(b'#')
            .add(b'<')
            .add(b'>');

        if let Some(response) = reqwest::blocking::get(format!(
            "https://www.google.com/transliterate?langpair=ja-Hira|ja&text={text}",
            text = percent_encoding::utf8_percent_encode(&mgr.status.buffer, QUERY)
        ))
        .and_then(|x| x.json::<Vec<(String, Vec<String>)>>())
        .ok()
        {
            mgr.status.candidates_idx = 0;
            mgr.status.candidates = response
                .into_iter()
                .map(|(_input, candidates)| HenkanCandidate {
                    candidates,
                    index: 0,
                })
                .collect();
        };
    }

    fn new_line_key(mgr: &mut KeyboardManager) {
        debug_assert!(mgr.status.buffer.is_empty());
        os::enter_enter();
    }

    fn kakutei_key(mgr: &mut KeyboardManager) {
        debug_assert!(!mgr.status.buffer.is_empty());
        mgr.flush()
    }

    fn backspace_key(mgr: &mut KeyboardManager) {
        if let Some(_) = mgr.status.buffer.pop() {
            if mgr.status.buffer.is_empty() {
                mgr.set_inputted_table();
            }
        } else {
            os::enter_backspace();
        }
    }

    fn space_key(mgr: &mut KeyboardManager) {
        if mgr.status.buffer.is_empty() {
            os::enter_char(' ');
        } else {
            mgr.status.buffer.push(' ');
        }
    }

    fn next_plane_key(mgr: &mut KeyboardManager) {
        mgr.move_to_next_plane()
    }

    fn sign_plane_key(mgr: &mut KeyboardManager) {
        mgr.swap_sign_plane()
    }
}

macro_rules! builtin_button {
    ($char: literal = $func: ident) => {
        CleKeyButton(&[CleKeyButtonAction {
            shows: $char,
            action: InputNextAction::Intrinsic(KeyboardManager::$func),
        }])
    };
}

impl<'ovr> KeyboardManager<'ovr> {
    fn set_plane(&mut self, table: &CleKeyInputTable<'static>) {
        use input_method::*;
        self.status.method.clone_from(table);

        self.status.method.table[6 * 8 + 6] = builtin_button!("⌫" = backspace_key);
        self.status.method.table[6 * 8 + 7] = builtin_button!("␣" = space_key);

        self.status.method.table[7 * 8 + 6] = builtin_button!("\u{1F310}" = next_plane_key); // 🌐
        self.status.method.table[7 * 8 + 7] = builtin_button!("#+=" = sign_plane_key);

        if self.status.buffer.is_empty() {
            self.set_inputted_table();
        } else {
            self.set_inputting_table();
        }
    }

    fn set_inputted_table(&mut self) {
        use input_method::*;
        self.status.method.table[5 * 8 + 6] = builtin_button!("Close" = close_key);
        self.status.method.table[5 * 8 + 7] = builtin_button!("⏎" = new_line_key);
    }

    fn set_inputting_table(&mut self) {
        use input_method::*;
        self.status.method.table[5 * 8 + 6] = builtin_button!("変換" = henkan_key);
        self.status.method.table[5 * 8 + 7] = builtin_button!("確定" = kakutei_key);
    }
}
