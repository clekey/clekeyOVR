use std::fmt::{Display, Formatter};
use std::path::Path;
use gl::types::GLuint;
use glam::Vec2;
use crate::{CleKeyConfig, LeftRight};
use super::*;

pub(in super) struct OVRController {
    _unused: ()
}

macro_rules! assume_used {
    ($($var: ident),*) => {
        let _ = ($($var,)*);
    };
}

impl OvrImpl for OVRController {
    fn new(_resources: &Path) -> Result<OVRController> {
        Ok(Self {_unused: ()})
    }

    fn load_config(&self, config: &CleKeyConfig) -> Result<()> {
        assume_used!(config);
        Ok(())
    }

    fn set_active_action_set(&self, kinds: impl IntoIterator<Item = ActionSetKind>) -> Result<()> {
        assume_used!(kinds);
        Ok(())
    }

    fn set_texture_impl(&self, texture: GLuint, handle: usize) -> Result<()> {
        assume_used!(texture, handle);
        Ok(())
    }

    fn hide_overlays(&self) -> Result<()> {
        Ok(())
    }

    fn close_center_overlay(&self) -> Result<()> {
        Ok(())
    }

    fn stick_pos(&self, hand: LeftRight) -> Result<Vec2> {
        assume_used!(hand);
        Ok(Vec2::default())
    }

    fn trigger_status(&self, hand: LeftRight) -> Result<bool> {
        assume_used!(hand);
        Ok(false)
    }

    fn play_haptics(
        &self,
        hand: LeftRight,
        start_seconds_from_now: f32,
        duration_seconds: f32,
        frequency: f32,
        amplitude: f32,
    ) -> Result<()> {
        assume_used!(hand, start_seconds_from_now, duration_seconds, frequency, amplitude);
        Ok(())
    }

    fn button_status(&self, button: ButtonKind) -> Result<bool> {
        assume_used!(button);
        Ok(false)
    }


    fn click_started(&self, button: ButtonKind) -> Result<bool> {
        assume_used!(button);
        Ok(false)
    }
}

#[derive(Debug)]
pub enum OVRError {}

impl Display for OVRError {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}