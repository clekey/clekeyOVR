use std::fmt::{Display, Formatter};
use std::path::Path;
use openvr::{cstr, VRActionHandle_t, VRActionSetHandle_t, VRContext};
use openvr::overlay::OwnedInVROverlay;
use crate::utils::{IntoStringLossy, ToCString};

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
    pub fn new(resources: &Path) -> Result<OVRController, OVRError> {
        let context = openvr::init(openvr::ApplicationType::Overlay)?;
        // load required components
        let overlay = context.overlay()?;
        let input = context.input()?;
        context.system()?;

        let path = resources.join("actions.json");

        input.set_action_manifest_path(path.into_string_lossy().to_c_string().as_c_str())?;

        let action_input_left_stick = input.get_action_handle(cstr!("/actions/input/in/left_stick"))?;
        let action_input_left_click = input.get_action_handle(cstr!("/actions/input/in/left_click"))?;
        let action_input_left_haptic = input.get_action_handle(cstr!("/actions/input/output/left_haptic"))?;
        let action_input_right_stick = input.get_action_handle(cstr!("/actions/input/in/right_stick"))?;
        let action_input_right_click = input.get_action_handle(cstr!("/actions/input/in/right_click"))?;
        let action_input_right_haptic = input.get_action_handle(cstr!("/actions/input/output/right_haptic"))?;
        let action_set_input = input.get_action_handle(cstr!("/actions/input"))?;

        let action_waiting_begin_input = input.get_action_handle(cstr!("/actions/waiting/in/begin_input"))?;
        let action_set_waiting = input.get_action_set_handle(cstr!("/actions/waiting"))?;

        let action_suspender_suspender = input.get_action_handle(cstr!("/actions/suspender/in/suspender"))?;
        let action_set_suspender = input.get_action_set_handle(cstr!("/actions/suspender"))?;

        let overlay_handles = [
            OwnedInVROverlay::new(overlay, cstr!("com.anatawa12.clekey-ovr.left"), cstr!("clekey-ovr left"))?,
            OwnedInVROverlay::new(overlay, cstr!("com.anatawa12.clekey-ovr.right"), cstr!("clekey-ovr right"))?,
            OwnedInVROverlay::new(overlay, cstr!("com.anatawa12.clekey-ovr.center"), cstr!("clekey-ovr center"))?,
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
