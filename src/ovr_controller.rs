use crate::{CleKeyConfig, HandInfo, KeyboardStatus, LeftRight, Vec2};
use gl::types::GLuint;
use std::f32::consts::PI;
use std::fmt;
use std::path::Path;

#[cfg(not(feature = "openvr"))]
mod mock;
#[cfg(not(feature = "openvr"))]
use mock as ovr;
use crate::input_method::HardKeyButton;

#[cfg(feature = "openvr")]
mod ovr;

pub type Result<T> = core::result::Result<T, OVRError>;

trait OvrImpl : Sized {
    type OverlayPlaneHandle: OverlayPlaneHandle;
    fn new(resources: &Path) -> Result<Self>;
    fn load_config(&self, config: &CleKeyConfig) -> Result<()>;
    fn set_active_action_set(&self, kinds: impl IntoIterator<Item = ActionSetKind>) -> Result<()>;
    fn plane_handle(&self, plane: OverlayPlane) -> &Self::OverlayPlaneHandle;
    fn stick_pos(&self, hand: LeftRight) -> Result<Vec2>;
    fn trigger_status(&self, hand: LeftRight) -> Result<bool>;
    fn play_haptics(
        &self,
        hand: LeftRight,
        start_seconds_from_now: f32,
        duration_seconds: f32,
        frequency: f32,
        amplitude: f32,
    ) -> Result<()>;
    fn hide_overlays(&self) -> Result<()>;
    fn close_center_overlay(&self) -> Result<()>;
    fn button_status(&self, button: ButtonKind) -> bool;
    fn click_started(&self, button: HardKeyButton) -> bool;
}

trait OverlayPlaneHandle {
    fn set_texture(&self, texture: GLuint) -> Result<()>;
    fn is_visible(&self) -> bool;
    fn show_overlay(&self) -> Result<()>;
    fn hide_overlay(&self) -> Result<()>;
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
    fn update_hand_status(&self, status: &mut HandInfo, hand: LeftRight) -> Result<()> {
        status.stick = self.stick_pos(hand)?;
        status.selection_old = status.selection;

        fn compute_angle(vec: Vec2) -> i8 {
            let a: f32 = vec.y.atan2(vec.x) * (4.0 / PI);
            let mut angle = a.round() as i8;
            return (angle + 2) & 7;
        }

        const LOWER_BOUND: f32 = 0.75 * 0.75;
        const UPPER_BOUND: f32 = 0.8 * 0.8;

        let len_sqrt = status.stick.length_squared();
        status.selection = if len_sqrt >= UPPER_BOUND {
            compute_angle(status.stick)
        } else if len_sqrt >= LOWER_BOUND && status.selection != -1 {
            compute_angle(status.stick)
        } else {
            -1
        };

        if status.selection != status.selection_old {
            self.play_haptics(hand, 0.0, 0.05, 1.0, 0.5)?;
        }

        status.clicking_old = status.clicking;
        status.clicking = self.trigger_status(hand)?;
        Ok(())
    }

    pub fn update_status(&self, status: &mut KeyboardStatus) -> Result<()> {
        self.update_hand_status(&mut status.left, LeftRight::Left)?;
        self.update_hand_status(&mut status.left, LeftRight::Left)?;
        Ok(())
    }

    pub fn show_overlay(&self, plane: OverlayPlane) -> Result<()> {
        Ok(self.main.plane_handle(plane).show_overlay()?)
    }

    pub fn hide_overlay(&self, plane: OverlayPlane) -> Result<()> {
        Ok(self.main.plane_handle(plane).hide_overlay()?)
    }

    pub fn draw_if_visible(&self, plane: OverlayPlane, renderer: impl FnOnce() -> GLuint) -> Result<()> {
        let handle = self.main.plane_handle(plane);
        if handle.is_visible() {
            handle.set_texture(renderer())?;
        }
        Ok(())
    }
}

macro_rules! trait_wrap {
    ($vis: vis fn $name: ident(&self, $($arg_n: ident: $arg_ty: ty),* $(,)?) -> $returns: ty; $($tt:tt)*) => {
        $vis fn $name(&self, $($arg_n: $arg_ty),*) -> $returns {
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
        Ok(Self { main: ovr::OVRController::new(resources)? })
    }

    trait_wrap! {
        pub fn load_config(&self, config: &CleKeyConfig) -> Result<()>;
        pub fn set_active_action_set(&self, kinds: impl IntoIterator<Item = ActionSetKind>) -> Result<()>;
        pub fn stick_pos(&self, hand: LeftRight) -> Result<Vec2>;
        pub fn trigger_status(&self, hand: LeftRight) -> Result<bool>;
        pub fn play_haptics(
            &self,
            hand: LeftRight,
            start_seconds_from_now: f32,
            duration_seconds: f32,
            frequency: f32,
            amplitude: f32,
        ) -> Result<()>;
        pub fn button_status(&self, button: ButtonKind) -> bool;
        pub fn click_started(&self, button: HardKeyButton) -> bool;
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ButtonKind {
    BeginInput,
    SuspendInput,
}

pub struct OVRError {
    main: ovr::OVRError,
}

impl <T> From<T> for OVRError where ovr::OVRError: From<T> {
    fn from(t: T) -> Self {
        Self { main: t.into() }
    }
}

impl fmt::Debug for OVRError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.main, f)
    }
}
