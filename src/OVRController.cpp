//
// Created by anatawa12 on 8/11/22.
//

#include "OVRController.h"

#ifdef WITH_OPEN_VR

#include <iostream>

void handle_input_err(vr::EVRInputError error) {
  if (error != vr::VRInputError_None) {
    std::cerr << "input error: " << error << std::endl;
  }
}

void handle_overlay_err(vr::EVROverlayError error) {
  if (error != vr::VROverlayError_None) {
    std::cerr << "input error (" << error << "): " << vr::VROverlay()->GetOverlayErrorNameFromEnum(error)
              << std::endl;
  }
}

bool init_ovr() {
  vr::HmdError err;
  vr::VR_Init(&err, vr::EVRApplicationType::VRApplication_Overlay);
  if (!vr::VROverlay()) {
    std::cerr << "error: " << vr::VR_GetVRInitErrorAsEnglishDescription(err) << std::endl;
    return false;
  }
  return true;
}

void shutdown_ovr() {
  vr::VR_Shutdown();
}

OVRController::OVRController() {
  // pre init
  action_left_stick = 0;
  action_left_click = 0;
  action_left_haptic = 0;
  action_right_stick = 0;
  action_right_click = 0;
  action_right_haptic = 0;
  action_set_input = 0;
  overlay_handle = 0;

  handle_input_err(vr::VRInput()->SetActionManifestPath(
      R"(C:\Users\anata\clekey-ovr-build\actions.json)"));

#define GetActionHandle(name) handle_input_err(vr::VRInput()->GetActionHandle("/actions/input/in/" #name, &action_##name))
  GetActionHandle(left_stick);
  GetActionHandle(left_click);
  GetActionHandle(left_haptic);
  GetActionHandle(right_stick);
  GetActionHandle(right_click);
  GetActionHandle(right_haptic);
  handle_input_err(vr::VRInput()->GetActionSetHandle("/actions/input", &action_set_input));
#undef GetActionHandle

  handle_overlay_err(vr::VROverlay()->CreateOverlay("com.anatawa12.clekey-ovr", "clekey-ovr", &overlay_handle));
  vr::VROverlay()->SetOverlayWidthInMeters(overlay_handle, 2);
  vr::VROverlay()->SetOverlayAlpha(overlay_handle, 1.0);

  std::cout << "action_left_stick:   " << action_left_stick << std::endl;
  std::cout << "action_left_click:   " << action_left_click << std::endl;
  std::cout << "action_left_haptic:  " << action_left_haptic << std::endl;
  std::cout << "action_right_stick:  " << action_right_stick << std::endl;
  std::cout << "action_right_click:  " << action_right_click << std::endl;
  std::cout << "action_right_haptic: " << action_right_haptic << std::endl;
  std::cout << "action_set_input:    " << action_set_input << std::endl;

  {
    vr::HmdMatrix34_t position = {};

    position.m[0][0] = 1;
    position.m[1][1] = 1;
    position.m[2][2] = 1;

    position.m[0][3] = 0;
    position.m[1][3] = 0;
    position.m[2][3] = -1.5;

    vr::VROverlay()->SetOverlayTransformTrackedDeviceRelative(
        overlay_handle,
        vr::k_unTrackedDeviceIndex_Hmd,
        &position);

    vr::VROverlay()->SetOverlayCurvature(overlay_handle, .2);
  }

  std::cout << "successfully launched" << std::endl;
}

void OVRController::tick(GLuint texture) const {
  vr::VRActiveActionSet_t action = {};
  action.ulActionSet = action_set_input;
  handle_input_err(vr::VRInput()->UpdateActionState(&action, sizeof(vr::VRActiveActionSet_t), 1));
  vr::InputAnalogActionData_t analog_data = {};
  handle_input_err(vr::VRInput()->GetAnalogActionData(action_left_stick, &analog_data, sizeof(analog_data),
                                                      vr::k_ulInvalidInputValueHandle));
  std::cout << "left input:  " << analog_data.bActive << ": "
            << analog_data.x << ", " << analog_data.y << std::endl;
  handle_input_err(vr::VRInput()->GetAnalogActionData(
      action_right_stick, &analog_data, sizeof(analog_data),
      vr::k_ulInvalidInputValueHandle));
  std::cout << "right input: " << analog_data.bActive << ": "
            << analog_data.x << ", " << analog_data.y << std::endl;
  vr::InputDigitalActionData_t digital_data = {};
  handle_input_err(vr::VRInput()->GetDigitalActionData(
      action_left_click, &digital_data, sizeof(digital_data),
      vr::k_ulInvalidInputValueHandle));
  std::cout << "left click:  " << digital_data.bActive << ": "
            << digital_data.bState << std::endl;
  handle_input_err(vr::VRInput()->GetDigitalActionData(
      action_right_click, &digital_data, sizeof(digital_data),
      vr::k_ulInvalidInputValueHandle));
  std::cout << "right click: " << digital_data.bActive << ": "
            << digital_data.bState << std::endl;

  vr::VROverlay()->ShowOverlay(overlay_handle);

  if (vr::VROverlay()->IsOverlayVisible(overlay_handle)) {

    vr::Texture_t vr_texture = {};

    vr_texture.handle = (void *) (uintptr_t) texture;
    vr_texture.eType = vr::TextureType_OpenGL;
    vr_texture.eColorSpace = vr::ColorSpace_Auto;

    handle_overlay_err(vr::VROverlay()->SetOverlayTexture(
        overlay_handle,
        &vr_texture));
  }
}

glm::vec2 OVRController::getStickPos(LeftRight hand) const {
  vr::InputAnalogActionData_t analog_data = {};
  handle_input_err(vr::VRInput()->GetAnalogActionData(
      hand == LeftRight::Left ? action_left_stick : action_right_stick,
      &analog_data, sizeof(analog_data),
      vr::k_ulInvalidInputValueHandle));
  return {analog_data.x, analog_data.y};
}

#else

// no openVR
bool init_ovr() {
    return true;
}

void shutdown_ovr() {
}

OVRController::OVRController() = default;
void OVRController::tick(GLuint texture) const {}

#endif
