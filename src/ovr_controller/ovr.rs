use super::*;
use crate::config::{OverlayPositionConfig, UIMode};
use crate::utils::ToCString;
use crate::{CleKeyConfig, LeftRight, Vec2};
use gl::types::GLuint;
use glam::Vec3;
use log::info;
use openvr::overlay::OwnedInVROverlay;
use openvr::{
    ColorSpace, OverlayTexture, TextureType, VRActionHandle_t, VRActionSetHandle_t,
    VRActiveActionSet_t, VRContext, cstr,
};
use serde_json::Value;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::path::Path;

pub(super) struct OVRController {
    // input
    action_input_left_stick: VRActionHandle_t,
    action_input_left_click: VRActionHandle_t,
    action_input_left_haptic: VRActionHandle_t,
    action_input_right_stick: VRActionHandle_t,
    action_input_right_click: VRActionHandle_t,
    action_input_right_haptic: VRActionHandle_t,
    action_set_input: VRActionSetHandle_t,

    // waiting
    action_waiting_begin_input: VRActionHandle_t,
    action_set_waiting: VRActionSetHandle_t,

    // suspender
    action_suspender_suspender: VRActionHandle_t,
    action_set_suspender: VRActionSetHandle_t,

    // this is unsafe: OwnedInVROverlay is actually OwnedInVROverlay<'self>
    // or OwnedInVROverlay<'context> but there's no such a representation so
    // I'm using 'static instead
    //
    // Because Rust drops the values in order of fields declared,
    // this value no longer live longer than context (unless this moves out)
    overlay_handles: [OwnedInVROverlay<'static>; 3],

    context: VRContext,
}

#[repr(transparent)]
pub struct OverlayPlaneHandleWrapper(OwnedInVROverlay<'static>);

impl OvrImpl for OVRController {
    type OverlayPlaneHandle = OverlayPlaneHandleWrapper;

    fn new(resources: &Path) -> Result<Self> {
        let context = openvr::init(openvr::ApplicationType::Overlay)?;
        // load required components
        let overlay = context.overlay()?;
        let input = context.input()?;
        context.system()?;

        let path = resources.join("actions.json");

        input.set_action_manifest_path(&path.to_c_string())?;

        let application = context.application()?;
        let manifest_template = resources.join("vrmanifest-template.json");
        let manifest = resources.join("vrmanifest.json");

        make_manifest(&manifest_template, &manifest);

        let manifest_cstr = manifest.to_c_string();
        if application.is_application_installed(cstr!("com.anatawa12.clekey_ovr")) {
            application.remove_application_manifest(&manifest_cstr)?;
        }
        application.add_application_manifest(&manifest_cstr, false)?;

        fn make_manifest(template: &Path, output: &Path) {
            let exe_path = std::env::current_exe()
                .expect("exe path")
                .to_string_lossy()
                .into_owned();

            let mut manifest_template: Value = serde_json::from_str(
                &std::fs::read_to_string(&template).expect("opening manifest template"),
            )
            .expect("reading template");

            *manifest_template
                .get_mut("applications")
                .and_then(|x| x.get_mut(0))
                .and_then(|x| x.get_mut("binary_path_windows"))
                .unwrap() = Value::String(exe_path);
            std::fs::write(output, serde_json::to_string(&manifest_template).unwrap())
                .expect("writing actual manifest");
        }

        let action_input_left_stick =
            input.get_action_handle(cstr!("/actions/input/in/left_stick"))?;
        let action_input_left_click =
            input.get_action_handle(cstr!("/actions/input/in/left_click"))?;
        let action_input_left_haptic =
            input.get_action_handle(cstr!("/actions/input/out/left_haptic"))?;
        let action_input_right_stick =
            input.get_action_handle(cstr!("/actions/input/in/right_stick"))?;
        let action_input_right_click =
            input.get_action_handle(cstr!("/actions/input/in/right_click"))?;
        let action_input_right_haptic =
            input.get_action_handle(cstr!("/actions/input/out/right_haptic"))?;
        let action_set_input = input.get_action_handle(cstr!("/actions/input"))?;

        let action_waiting_begin_input =
            input.get_action_handle(cstr!("/actions/waiting/in/begin_input"))?;
        let action_set_waiting = input.get_action_set_handle(cstr!("/actions/waiting"))?;

        let action_suspender_suspender =
            input.get_action_handle(cstr!("/actions/suspender/in/suspender"))?;
        let action_set_suspender = input.get_action_set_handle(cstr!("/actions/suspender"))?;

        let overlay_handles = [
            OwnedInVROverlay::new(
                overlay,
                cstr!("com.anatawa12.clekey-ovr.left"),
                cstr!("clekey-ovr left"),
            )?,
            OwnedInVROverlay::new(
                overlay,
                cstr!("com.anatawa12.clekey-ovr.right"),
                cstr!("clekey-ovr right"),
            )?,
            OwnedInVROverlay::new(
                overlay,
                cstr!("com.anatawa12.clekey-ovr.center"),
                cstr!("clekey-ovr center"),
            )?,
        ];

        info!("action_left_stick:          {}", action_input_left_stick);
        info!("action_left_click:          {}", action_input_left_click);
        info!("action_left_haptic:         {}", action_input_left_haptic);
        info!("action_right_stick:         {}", action_input_right_stick);
        info!("action_right_click:         {}", action_input_right_click);
        info!("action_right_haptic:        {}", action_input_right_haptic);
        info!("action_set_input:           {}", action_set_input);
        info!("action_waiting_begin_input: {}", action_waiting_begin_input);
        info!("action_set_waiting:         {}", action_set_waiting);
        info!("action_suspender_suspender: {}", action_suspender_suspender);
        info!("action_set_suspender:       {}", action_set_suspender);

        Ok(Self {
            action_input_left_stick,
            action_input_left_click,
            action_input_left_haptic,
            action_input_right_stick,
            action_input_right_click,
            action_input_right_haptic,
            action_set_input,
            action_waiting_begin_input,
            action_set_waiting,
            action_suspender_suspender,
            action_set_suspender,
            overlay_handles: unsafe { std::mem::transmute(overlay_handles) },
            context,
        })
    }

    fn load_config(&self, config: &CleKeyConfig) -> Result<()> {
        fn overlay_position_matrix(yaw: f32, pitch: f32, distance: f32) -> openvr::HmdMatrix34_t {
            let mat = glam::Mat4::from_rotation_y(yaw.to_radians())
                * glam::Mat4::from_rotation_x(pitch.to_radians())
                * glam::Mat4::from_translation(Vec3::new(0.0, 0.0, -distance));

            let mat = mat.transpose();
            let cols = mat.to_cols_array_2d();
            openvr::HmdMatrix34_t {
                m: [cols[0], cols[1], cols[2]],
            }
        }

        fn load(handle: &OwnedInVROverlay, config: &OverlayPositionConfig) -> Result<()> {
            handle.set_overlay_width_in_meters(config.width_radio * config.distance)?;
            handle.set_overlay_alpha(1.0)?;
            handle.set_overlay_transform_tracked_device_relative(
                0,
                &overlay_position_matrix(config.yaw, config.pitch, config.distance),
            )?;
            Ok(())
        }

        match config.ui_mode {
            UIMode::TwoRing => {
                load(
                    &self.overlay_handles[0],
                    &config.two_ring.left_ring.position,
                )?;
                load(
                    &self.overlay_handles[1],
                    &config.two_ring.right_ring.position,
                )?;
                load(
                    &self.overlay_handles[2],
                    &config.two_ring.completion.position,
                )?;
            }
            UIMode::OneRing => {
                load(&self.overlay_handles[0], &config.one_ring.ring.position)?;
                load(
                    &self.overlay_handles[2],
                    &config.one_ring.completion.position,
                )?;
            }
        }
        Ok(())
    }

    fn set_active_action_set(&self, kinds: impl IntoIterator<Item = ActionSetKind>) {
        fn as_vr_action_set(c: &OVRController, kind: ActionSetKind) -> VRActiveActionSet_t {
            match kind {
                ActionSetKind::Input => VRActiveActionSet_t {
                    ulActionSet: c.action_set_input,
                    ulRestrictedToDevice: 0,
                    ulSecondaryActionSet: 0,
                    unPadding: 0,
                    nPriority: 0x01000000,
                },
                ActionSetKind::Waiting => VRActiveActionSet_t {
                    ulActionSet: c.action_set_waiting,
                    ulRestrictedToDevice: 0,
                    ulSecondaryActionSet: 0,
                    unPadding: 0,
                    nPriority: 0,
                },
                ActionSetKind::Suspender => VRActiveActionSet_t {
                    ulActionSet: c.action_set_suspender,
                    ulRestrictedToDevice: 0,
                    ulSecondaryActionSet: 0,
                    unPadding: 0,
                    nPriority: 0x01000000,
                },
            }
        }

        let sets = kinds
            .into_iter()
            .map(|x| as_vr_action_set(self, x))
            .collect::<Vec<_>>();
        self.context
            .input()
            .expect("input")
            .update_action_state(&sets)
            .expect("update_action_state");
    }

    fn plane_handle(&self, plane: OverlayPlane) -> &Self::OverlayPlaneHandle {
        unsafe {
            // SAFETY: OverlayPlaneHandleWrapper is transparent
            std::mem::transmute(&self.overlay_handles[plane as usize])
        }
    }

    fn stick_pos(&self, hand: LeftRight) -> Vec2 {
        let action = match hand {
            LeftRight::Left => self.action_input_left_stick,
            LeftRight::Right => self.action_input_right_stick,
        };
        let data = self
            .context
            .input()
            .expect("inputs")
            .get_analog_action_data(action, 0)
            .expect("get_analog_action_data");
        Vec2::new(data.x, data.y)
    }

    fn trigger_status(&self, hand: LeftRight) -> bool {
        let action = match hand {
            LeftRight::Left => self.action_input_left_click,
            LeftRight::Right => self.action_input_right_click,
        };
        let data = self
            .context
            .input()
            .expect("inputs")
            .get_digital_action_data(action, 0)
            .expect("get_digital_action_data");
        data.bState
    }

    fn play_haptics(
        &self,
        hand: LeftRight,
        start_seconds_from_now: f32,
        duration_seconds: f32,
        frequency: f32,
        amplitude: f32,
    ) -> () {
        let action = match hand {
            LeftRight::Left => self.action_input_left_haptic,
            LeftRight::Right => self.action_input_right_haptic,
        };

        self.context
            .input()
            .expect("inputs")
            .trigger_haptic_vibration_action(
                action,
                start_seconds_from_now,
                duration_seconds,
                frequency,
                amplitude,
                0,
            )
            .expect("trigger_haptic_vibration_action")
    }

    fn button_status(&self, button: ButtonKind) -> bool {
        let action = match button {
            //ButtonKind::BeginInput => self.action_waiting_begin_input,
            ButtonKind::SuspendInput => self.action_suspender_suspender,
        };
        self.context
            .input()
            .expect("inputs")
            .get_digital_action_data(action, 0)
            .unwrap_or_else(|e| panic!("getting button status {:?}: {:?}", button, e))
            .bState
    }

    fn click_started(&self, button: HardKeyButton) -> bool {
        let action = match button {
            HardKeyButton::CloseButton => self.action_waiting_begin_input,
        };
        let data = self
            .context
            .input()
            .expect("inputs")
            .get_digital_action_data(action, 0)
            .unwrap_or_else(|e| panic!("getting button status {:?}: {:?}", button, e));
        data.bState && data.bChanged
    }
}

impl OverlayPlaneHandle for OverlayPlaneHandleWrapper {
    fn set_texture(&self, texture: GLuint) {
        self.0
            .set_overlay_texture(OverlayTexture {
                handle: texture as usize as *mut c_void,
                tex_type: TextureType::OpenGL,
                color_space: ColorSpace::Auto,
            })
            .expect("set_overlay_texture")
    }

    fn is_visible(&self) -> bool {
        self.0.is_overlay_visible()
    }

    fn show_overlay(&self) {
        self.0.show_overlay().expect("show_overlay")
    }

    fn hide_overlay(&self) {
        self.0.hide_overlay().expect("hide_overlay")
    }
}

#[derive(Debug)]
pub enum OVRError {
    Init(openvr::InitError),
    Input(openvr::InputError),
    Overlay(openvr::OverlayError),
    Application(openvr::ApplicationError),
}

impl Display for OVRError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OVRError::Init(err) => Display::fmt(err, f),
            OVRError::Input(err) => Display::fmt(err, f),
            OVRError::Overlay(err) => Display::fmt(err, f),
            OVRError::Application(err) => Display::fmt(err, f),
        }
    }
}

impl From<openvr::InitError> for OVRError {
    fn from(v: openvr::InitError) -> Self {
        OVRError::Init(v)
    }
}

impl From<openvr::InputError> for OVRError {
    fn from(v: openvr::InputError) -> Self {
        OVRError::Input(v)
    }
}

impl From<openvr::OverlayError> for OVRError {
    fn from(v: openvr::OverlayError) -> Self {
        OVRError::Overlay(v)
    }
}

impl From<openvr::ApplicationError> for OVRError {
    fn from(v: openvr::ApplicationError) -> Self {
        OVRError::Application(v)
    }
}
