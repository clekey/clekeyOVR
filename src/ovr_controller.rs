use crate::config::OverlayPositionConfig;
use crate::utils::{IntoStringLossy, ToCString};
use crate::{CleKeyConfig, HandInfo, KeyboardStatus, LeftRight, Vec2};
use gl::types::GLuint;
use glam::Vec3;
use openvr::overlay::OwnedInVROverlay;
use openvr::{
    cstr, ColorSpace, OverlayTexture, TextureType, VRActionHandle_t,
    VRActionSetHandle_t, VRActiveActionSet_t, VRContext,
};
use std::f32::consts::PI;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::path::Path;

pub type Result<T> = core::result::Result<T, OVRError>;

pub struct OVRController {
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

impl OVRController {
    pub fn new(resources: &Path) -> Result<OVRController> {
        let context = openvr::init(openvr::ApplicationType::Overlay)?;
        // load required components
        let overlay = context.overlay()?;
        let input = context.input()?;
        context.system()?;

        let path = resources.join("actions.json");

        input.set_action_manifest_path(path.into_string_lossy().to_c_string().as_c_str())?;

        let action_input_left_stick =
            input.get_action_handle(cstr!("/actions/input/in/left_stick"))?;
        let action_input_left_click =
            input.get_action_handle(cstr!("/actions/input/in/left_click"))?;
        let action_input_left_haptic =
            input.get_action_handle(cstr!("/actions/input/output/left_haptic"))?;
        let action_input_right_stick =
            input.get_action_handle(cstr!("/actions/input/in/right_stick"))?;
        let action_input_right_click =
            input.get_action_handle(cstr!("/actions/input/in/right_click"))?;
        let action_input_right_haptic =
            input.get_action_handle(cstr!("/actions/input/output/right_haptic"))?;
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

        println!("action_left_stick:          {}", action_input_left_stick);
        println!("action_left_click:          {}", action_input_left_click);
        println!("action_left_haptic:         {}", action_input_left_haptic);
        println!("action_right_stick:         {}", action_input_right_stick);
        println!("action_right_click:         {}", action_input_right_click);
        println!("action_right_haptic:        {}", action_input_right_haptic);
        println!("action_set_input:           {}", action_set_input);
        println!("action_waiting_begin_input: {}", action_waiting_begin_input);
        println!("action_set_waiting:         {}", action_set_waiting);
        println!("action_suspender_suspender: {}", action_suspender_suspender);
        println!("action_set_suspender:       {}", action_set_suspender);

        Ok(OVRController {
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

    pub fn load_config(&self, config: &CleKeyConfig) -> Result<()> {
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
        load(&self.overlay_handles[0], &config.left_ring.position);
        load(&self.overlay_handles[1], &config.right_ring.position);
        load(&self.overlay_handles[2], &config.completion.position);
        Ok(())
    }

    pub(super) fn as_vr_action_set(&self, kind: ActionSetKind) -> VRActiveActionSet_t {
        match kind {
            ActionSetKind::Input => VRActiveActionSet_t {
                ulActionSet: self.action_set_input,
                ulRestrictedToDevice: 0,
                ulSecondaryActionSet: 0,
                unPadding: 0,
                nPriority: 0x01000000,
            },
            ActionSetKind::Waiting => VRActiveActionSet_t {
                ulActionSet: self.action_set_waiting,
                ulRestrictedToDevice: 0,
                ulSecondaryActionSet: 0,
                unPadding: 0,
                nPriority: 0,
            },
            ActionSetKind::Suspender => VRActiveActionSet_t {
                ulActionSet: self.action_set_suspender,
                ulRestrictedToDevice: 0,
                ulSecondaryActionSet: 0,
                unPadding: 0,
                nPriority: 0x01000000,
            },
        }
    }

    pub fn set_active_action_set(
        &self,
        kinds: impl IntoIterator<Item = ActionSetKind>,
    ) -> Result<()> {
        let sets = kinds
            .into_iter()
            .map(|x| self.as_vr_action_set(x))
            .collect::<Vec<_>>();
        self.context
            .input()
            .expect("input")
            .update_action_state(&sets)?;
        Ok(())
    }

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

    fn set_texture_impl(&self, texture: GLuint, handle: usize) -> Result<()> {
        let handle = &self.overlay_handles[handle];
        handle.show_overlay()?;
        if handle.is_overlay_visible() {
            handle.set_overlay_texture(OverlayTexture {
                handle: texture as usize as *mut c_void,
                tex_type: TextureType::OpenGL,
                color_space: ColorSpace::Auto,
            })?;
        }
        Ok(())
    }

    pub fn set_texture(&self, texture: GLuint, side: LeftRight) -> Result<()> {
        self.set_texture_impl(texture, side as usize)
    }

    pub fn set_center_texture(&self, texture: GLuint) -> Result<()> {
        self.set_texture_impl(texture, 3)
    }

    pub fn hide_overlays(&self) -> Result<()> {
        for x in &self.overlay_handles {
            x.hide_overlay()?
        }
        Ok(())
    }

    pub fn close_center_overlay(&self) -> Result<()> {
        Ok(self.overlay_handles[3].hide_overlay()?)
    }

    pub fn stick_pos(&self, hand: LeftRight) -> Result<Vec2> {
        let action = match hand {
            LeftRight::Left => self.action_input_left_stick,
            LeftRight::Right => self.action_input_right_stick,
        };
        let data = self.context
            .input()
            .expect("inputs")
            .get_analog_action_data(action, 0)?;
        Ok(Vec2::new(data.x, data.y))
    }

    pub fn trigger_status(&self, hand: LeftRight) -> Result<bool> {
        let action = match hand {
            LeftRight::Left => self.action_input_left_click,
            LeftRight::Right => self.action_input_right_click,
        };
let  data=         self.context
            .input()
            .expect("inputs")
            .get_digital_action_data(action, 0)?;
        Ok(data.bState)
    }

    pub fn play_haptics(
        &self,
        hand: LeftRight,
        start_seconds_from_now: f32,
        duration_seconds: f32,
        frequency: f32,
        amplitude: f32,
    ) -> Result<()> {
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
                0
            )
            .map_err(Into::into)
    }

    pub fn button_status(&self, button: ButtonKind) -> Result<bool> {
        let action = match button {
            ButtonKind::BeginInput => self.action_waiting_begin_input,
            ButtonKind::SuspendInput => self.action_suspender_suspender,
        };
        let  data=         self.context
            .input()
            .expect("inputs")
            .get_digital_action_data(action, 0)?;
        Ok(data.bState)
    }


    pub fn click_started(&self, button: ButtonKind) -> Result<bool> {
        let action = match button {
            ButtonKind::BeginInput => self.action_waiting_begin_input,
            ButtonKind::SuspendInput => self.action_suspender_suspender,
        };
        let  data=         self.context
            .input()
            .expect("inputs")
            .get_digital_action_data(action, 0)?;
        Ok(data.bState && data.bChanged)
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

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ButtonKind {
    BeginInput,
    SuspendInput,
}

#[derive(Debug)]
pub enum OVRError {
    Init(openvr::InitError),
    Input(openvr::InputError),
    Overlay(openvr::OverlayError),
}

impl Display for OVRError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OVRError::Init(err) => Display::fmt(err, f),
            OVRError::Input(err) => Display::fmt(err, f),
            OVRError::Overlay(err) => Display::fmt(err, f),
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
