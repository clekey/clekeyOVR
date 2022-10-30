use super::*;
use crate::{CleKeyConfig, LeftRight};
use gl::types::GLuint;
use glam::Vec2;
use std::fmt::{Display, Formatter};
use std::path::Path;

pub(super) struct OVRController {
    _unused: (),
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
        Ok(Self { _unused: () })
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
        assume_used!(hand);
        Vec2::new(1.0, 0.0)
    }

    fn trigger_status(&self, hand: LeftRight) -> bool {
        assume_used!(hand);
        false
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
        assume_used!(button);
        false
    }

    fn click_started(&self, button: HardKeyButton) -> bool {
        assume_used!(button);
        false
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
