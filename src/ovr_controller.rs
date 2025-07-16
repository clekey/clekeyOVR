use crate::{CleKeyConfig, HandInfo, KeyboardStatus, LeftRight, Vec2};
use gl::types::GLuint;
use std::f32::consts::PI;
use std::fmt;
use std::path::Path;

#[cfg(not(feature = "openvr"))]
mod mock;
use crate::input_method::HardKeyButton;
#[cfg(not(feature = "openvr"))]
use mock as ovr;

#[cfg(feature = "openvr")]
mod ovr;

pub type Result<T> = core::result::Result<T, OVRError>;

trait OvrImpl: Sized {
    type OverlayPlaneHandle: OverlayPlaneHandle;
    fn new(resources: &Path) -> Result<Self>;
    fn load_config(&self, config: &CleKeyConfig) -> Result<()>;
    fn set_active_action_set(&self, kinds: impl IntoIterator<Item = ActionSetKind>);
    fn plane_handle(&self, plane: OverlayPlane) -> &Self::OverlayPlaneHandle;
    fn stick_pos(&self, hand: LeftRight) -> Vec2;
    fn trigger_status(&self, hand: LeftRight) -> bool;
    fn play_haptics(
        &self,
        hand: LeftRight,
        start_seconds_from_now: f32,
        duration_seconds: f32,
        frequency: f32,
        amplitude: f32,
    );
    fn button_status(&self, button: ButtonKind) -> bool;
    fn click_started(&self, button: HardKeyButton) -> bool;
}

trait OverlayPlaneHandle {
    fn set_texture(&self, texture: GLuint);
    fn is_visible(&self) -> bool;
    fn show_overlay(&self);
    fn hide_overlay(&self);
}

pub struct OVRController {
    main: ovr::OVRController,
}

#[derive(Copy, Clone)]
pub enum OverlayPlane {
    Left,
    Right,
    Center,
}

impl OverlayPlane {
    pub const VALUES: [OverlayPlane; 3] = [
        OverlayPlane::Left,
        OverlayPlane::Right,
        OverlayPlane::Center,
    ];
}

impl From<LeftRight> for OverlayPlane {
    fn from(side: LeftRight) -> Self {
        match side {
            LeftRight::Left => OverlayPlane::Left,
            LeftRight::Right => OverlayPlane::Right,
        }
    }
}

impl OVRController {
    fn update_hand_status(&self, status: &mut HandInfo, hand: LeftRight, clicking: bool) {
        status.stick = self.stick_pos(hand);
        status.selection_old = status.selection;

        fn compute_angle(vec: Vec2) -> i8 {
            let mut a: f32 = vec.y.atan2(vec.x);
            // (-pi, pi]
            a *= -4.0 / PI;
            // [-4, 4)
            a += 2.5;
            // [-1.5, 6.5)
            if a < 0.0 {
                a += 8.0
            }
            // [0, 8)
            return a.floor() as i8;
        }

        const LOWER_BOUND: f32 = 0.75 * 0.75;
        const UPPER_BOUND: f32 = 0.8 * 0.8;

        let len_sqrt = status.stick.length_squared();
        status.selection = if clicking {
            // do not change if clicking
            status.selection
        } else if len_sqrt >= UPPER_BOUND {
            compute_angle(status.stick)
        } else if len_sqrt >= LOWER_BOUND && status.selection != -1 {
            compute_angle(status.stick)
        } else {
            -1
        };

        if status.selection != status.selection_old {
            self.play_haptics(hand, 0.0, 0.05, 1.0, 0.5);
        }

        status.clicking_old = status.clicking;
        status.clicking = self.trigger_status(hand);
    }

    pub fn update_status(&self, status: &mut KeyboardStatus) {
        let clicking = status.clicking();
        self.update_hand_status(&mut status.left, LeftRight::Left, clicking);
        self.update_hand_status(&mut status.right, LeftRight::Right, clicking);
    }

    pub fn show_overlay(&self, plane: OverlayPlane) {
        self.main.plane_handle(plane).show_overlay();
    }

    pub fn hide_overlay(&self, plane: OverlayPlane) {
        self.main.plane_handle(plane).hide_overlay();
    }

    pub fn hide_all_overlay(&self) {
        for x in OverlayPlane::VALUES {
            self.hide_overlay(x);
        }
    }

    pub fn draw_if_visible(&self, plane: OverlayPlane, renderer: impl FnOnce() -> GLuint) {
        let handle = self.main.plane_handle(plane);
        if handle.is_visible() {
            handle.set_texture(renderer());
        }
    }
}

macro_rules! trait_wrap {
    ($vis: vis fn $name: ident(&self, $($arg_n: ident: $arg_ty: ty),* $(,)?)$( -> $returns: ty)?; $($tt:tt)*) => {
        $vis fn $name(&self, $($arg_n: $arg_ty),*)$( -> $returns)? {
            self.main.$name($($arg_n),*)
        }
        trait_wrap!{$($tt)*}
    };

    () => {
    };
}

impl OVRController {
    // trait wrappers
    pub fn new(resources: &Path) -> Result<OVRController> {
        Ok(Self {
            main: ovr::OVRController::new(resources)?,
        })
    }

    trait_wrap! {
        pub fn load_config(&self, config: &CleKeyConfig) -> Result<()>;
        pub fn set_active_action_set(&self, kinds: impl IntoIterator<Item = ActionSetKind>);
        pub fn stick_pos(&self, hand: LeftRight) -> Vec2;
        pub fn trigger_status(&self, hand: LeftRight) -> bool;
        pub fn play_haptics(
            &self,
            hand: LeftRight,
            start_seconds_from_now: f32,
            duration_seconds: f32,
            frequency: f32,
            amplitude: f32,
        ) -> ();
        pub fn button_status(&self, button: ButtonKind) -> bool;
        pub fn click_started(&self, button: HardKeyButton) -> bool;
    }
}

// mock-only debug_control
#[cfg(all(feature = "debug_control", not(feature = "openvr")))]
impl OVRController {
    pub(crate) fn accept_debug_control(&self, event: glfw::WindowEvent) {
        self.main.accept_debug_control(event);
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ActionSetKind {
    // action set have sticks
    Input,
    // action set for waiting: button to turn on keyboard
    Waiting,
    // action set for waiting: button to turn on clekey
    Suspender,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum ButtonKind {
    //BeginInput,
    SuspendInput,
}

pub struct OVRError {
    main: ovr::OVRError,
}

impl<T> From<T> for OVRError
where
    ovr::OVRError: From<T>,
{
    fn from(t: T) -> Self {
        Self { main: t.into() }
    }
}

impl fmt::Debug for OVRError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.main, f)
    }
}
