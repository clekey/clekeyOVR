use super::*;
use crate::{CleKeyConfig, LeftRight};
use gl::types::GLuint;
use glam::Vec2;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::Path;

pub(super) struct OVRController {
    inner: UnsafeCell<Mocked>,
}

macro_rules! assume_used {
    ($($var: ident),*) => {
        let _ = ($($var,)*);
    };
}

pub(super) struct MockHandle;

impl OvrImpl for OVRController {
    type OverlayPlaneHandle = MockHandle;

    fn new(_resources: &Path) -> Result<OVRController> {
        Ok(Self {
            inner: UnsafeCell::new(Mocked::new()),
        })
    }

    fn load_config(&self, config: &CleKeyConfig) -> Result<()> {
        assume_used!(config);
        Ok(())
    }

    fn set_active_action_set(&self, kinds: impl IntoIterator<Item = ActionSetKind>) {
        assume_used!(kinds);
    }

    fn plane_handle(&self, plane: OverlayPlane) -> &Self::OverlayPlaneHandle {
        assume_used!(plane);
        &MockHandle
    }

    fn stick_pos(&self, hand: LeftRight) -> Vec2 {
        self.inner().stick(hand)
    }

    fn trigger_status(&self, hand: LeftRight) -> bool {
        self.inner().trigger(hand)
    }

    fn play_haptics(
        &self,
        hand: LeftRight,
        start_seconds_from_now: f32,
        duration_seconds: f32,
        frequency: f32,
        amplitude: f32,
    ) {
        assume_used!(
            hand,
            start_seconds_from_now,
            duration_seconds,
            frequency,
            amplitude
        );
    }

    fn button_status(&self, button: ButtonKind) -> bool {
        self.inner().button(button)
    }

    fn click_started(&self, button: HardKeyButton) -> bool {
        assume_used!(button);
        false
    }
}

impl OVRController {
    #[cfg(feature = "debug_control")]
    pub(crate) fn accept_debug_control(&self, event: glfw::WindowEvent) {
        unsafe { &mut *self.inner.get() }.accept_debug_control(event);
    }

    fn inner(&self) -> &Mocked {
        unsafe { &*self.inner.get() }
    }
}

#[derive(Default)]
struct Mocked {
    sticks: HashMap<LeftRight, Vec2>,
    triggers: HashMap<LeftRight, bool>,
    buttons: HashMap<ButtonKind, bool>,
}

impl Mocked {
    fn new() -> Mocked {
        Default::default()
    }

    fn stick(&self, hand: LeftRight) -> Vec2 {
        self.sticks.get(&hand).copied().unwrap_or_default()
    }

    fn trigger(&self, hand: LeftRight) -> bool {
        self.triggers.get(&hand).copied().unwrap_or_default()
    }

    fn button(&self, hand: ButtonKind) -> bool {
        self.buttons.get(&hand).copied().unwrap_or_default()
    }

    #[cfg(feature = "debug_control")]
    pub(crate) fn accept_debug_control(&mut self, event: glfw::WindowEvent) {
        //info!("key event: {:?}", event);
        use glfw::WindowEvent;
        match event {
            WindowEvent::Key(key, _, action, _) => {
                use glfw::Action::*;
                use glfw::Key::*;
                use LeftRight::{Left, Right};
                match (key, action) {
                    // following for left stick
                    //ERT
                    //D G
                    //XCV
                    // release to reset to 0
                    (R | T | G | V | C | X | D | E, Release) => {
                        self.sticks.insert(Left, Vec2::new(0.0, 0.0));
                    }
                    // press & continue to tilt
                    (R, _) => {
                        self.sticks.insert(Left, Vec2::new(0.0, 1.0));
                    }
                    (T, _) => {
                        self.sticks.insert(Left, Vec2::new(0.7, 0.7));
                    }
                    (G, _) => {
                        self.sticks.insert(Left, Vec2::new(1.0, 0.0));
                    }
                    (V, _) => {
                        self.sticks.insert(Left, Vec2::new(0.7, -0.7));
                    }
                    (C, _) => {
                        self.sticks.insert(Left, Vec2::new(0.0, -1.0));
                    }
                    (X, _) => {
                        self.sticks.insert(Left, Vec2::new(-0.7, -0.7));
                    }
                    (D, _) => {
                        self.sticks.insert(Left, Vec2::new(-1.0, 0.0));
                    }
                    (E, _) => {
                        self.sticks.insert(Left, Vec2::new(-0.7, 0.7));
                    }

                    // F for left trigger
                    (F, Press) => {
                        self.triggers.insert(Left, true);
                    }
                    (F, Release) => {
                        self.triggers.insert(Left, false);
                    }

                    // following for right stick
                    //YUI
                    //H K
                    //BNM
                    // release to reset to 0
                    (U | I | K | M | N | B | H | Y, Release) => {
                        self.sticks.insert(Right, Vec2::new(0.0, 0.0));
                    }
                    // press & continue to tilt
                    (U, _) => {
                        self.sticks.insert(Right, Vec2::new(0.0, 1.0));
                    }
                    (I, _) => {
                        self.sticks.insert(Right, Vec2::new(0.7, 0.7));
                    }
                    (K, _) => {
                        self.sticks.insert(Right, Vec2::new(1.0, 0.0));
                    }
                    (M, _) => {
                        self.sticks.insert(Right, Vec2::new(0.7, -0.7));
                    }
                    (N, _) => {
                        self.sticks.insert(Right, Vec2::new(0.0, -1.0));
                    }
                    (B, _) => {
                        self.sticks.insert(Right, Vec2::new(-0.7, -0.7));
                    }
                    (H, _) => {
                        self.sticks.insert(Right, Vec2::new(-1.0, 0.0));
                    }
                    (Y, _) => {
                        self.sticks.insert(Right, Vec2::new(-0.7, 0.7));
                    }

                    // J for left trigger
                    (J, Press) => {
                        self.triggers.insert(Right, true);
                    }
                    (J, Release) => {
                        self.triggers.insert(Right, false);
                    }

                    _ => (),
                };
            }
            _ => {}
        };
    }
}

impl OverlayPlaneHandle for MockHandle {
    fn set_texture(&self, texture: GLuint) {
        assume_used!(texture);
    }

    fn is_visible(&self) -> bool {
        true
    }

    fn show_overlay(&self) {}

    fn hide_overlay(&self) {}
}

#[derive(Debug)]
pub enum OVRError {}

impl Display for OVRError {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}
