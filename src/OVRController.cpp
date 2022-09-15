//
// Created by anatawa12 on 8/11/22.
//

#include "OVRController.h"

#ifdef WITH_OPEN_VR

#include <iostream>
#include <filesystem>
#include "glm/gtc/constants.hpp"
#include "glm/gtx/transform.hpp"
#include "glm/gtx/vector_angle.hpp"

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

template<typename T>
const T* asPtr(const T& value) {
  return &value;
}

glm::mat4x3 overlayPositionMatrix(glm::vec3 position) {
  auto axis = glm::normalize(glm::cross(position, {0.0f, 0.0f, -1.0f}));
  auto angle = -glm::orientedAngle(glm::normalize(position), {0.0f, 0.0f, -1.0f}, axis);

  glm::mat4x4 matrix = glm::mat4x4(1);
  matrix = angle == 0 ? matrix : glm::rotate(matrix, angle, axis);;
  matrix = glm::translate(matrix, {0.0f, 0.0f, -1.5f});

  return matrix;
}

inline vr::HmdMatrix34_t toVR(const glm::mat4x3& mat) {
  return {{
              {mat[0][0], mat[1][0], mat[2][0], mat[3][0]},
              {mat[0][1], mat[1][1], mat[2][1], mat[3][1]},
              {mat[0][2], mat[1][2], mat[2][2], mat[3][2]},
  }};
}

OVRController::OVRController() { // NOLINT(cppcoreguidelines-pro-type-member-init)
  std::filesystem::path path = std::filesystem::current_path() / "actions.json";

  handle_input_err(vr::VRInput()->SetActionManifestPath(path.string().c_str()));

#define GetActionHandle(name) handle_input_err(vr::VRInput()->GetActionHandle("/actions/input/in/" #name, &action_##name))
  GetActionHandle(left_stick);
  GetActionHandle(left_click);
  GetActionHandle(left_haptic);
  GetActionHandle(right_stick);
  GetActionHandle(right_click);
  GetActionHandle(right_haptic);
  handle_input_err(vr::VRInput()->GetActionSetHandle("/actions/input", &action_set_input));
#undef GetActionHandle

  handle_overlay_err(vr::VROverlay()->CreateOverlay("com.anatawa12.clekey-ovr.left", "clekey-ovr left", &overlay_handles[0]));
  handle_overlay_err(vr::VROverlay()->CreateOverlay("com.anatawa12.clekey-ovr.right", "clekey-ovr right", &overlay_handles[1]));
  for (auto &overlay_handle: overlay_handles) {
    vr::VROverlay()->SetOverlayWidthInMeters(overlay_handle, .5);
    vr::VROverlay()->SetOverlayAlpha(overlay_handle, 1.0);
  }

  std::cout << "action_left_stick:   " << action_left_stick << std::endl;
  std::cout << "action_left_click:   " << action_left_click << std::endl;
  std::cout << "action_left_haptic:  " << action_left_haptic << std::endl;
  std::cout << "action_right_stick:  " << action_right_stick << std::endl;
  std::cout << "action_right_click:  " << action_right_click << std::endl;
  std::cout << "action_right_haptic: " << action_right_haptic << std::endl;
  std::cout << "action_set_input:    " << action_set_input << std::endl;

  {
    vr::VROverlay()->SetOverlayTransformTrackedDeviceRelative(
        overlay_handles[0],
        vr::k_unTrackedDeviceIndex_Hmd,
        asPtr(toVR(overlayPositionMatrix({-0.65f, -0.5f, -1.5f}))));

    vr::VROverlay()->SetOverlayTransformTrackedDeviceRelative(
        overlay_handles[1],
        vr::k_unTrackedDeviceIndex_Hmd,
        asPtr(toVR(overlayPositionMatrix({+0.65f, -0.5f, -1.5f}))));
  }

  std::cout << "successfully launched" << std::endl;
}

int8_t computeAngle(const glm::vec2 &stick) {
  float angleF = -std::atan2(stick.y, stick.x) / (glm::pi<float>() / 4);
  auto angle = int8_t(std::round(angleF));
  angle += 2;
  angle &= 7;
  return angle;
}

void updateHand(const OVRController &controller, AppStatus &status, LeftRight hand) {
  HandInfo& handInfo = status.getControllerInfo(hand);

  // first, set stick
  handInfo.stick = controller.getStickPos(hand);

  // then, set selection
  float lenSqrt = glm::dot(handInfo.stick, handInfo.stick);
  if (lenSqrt >= 0.8 * 0.8) {
    handInfo.selection = computeAngle(handInfo.stick);
  } else if (lenSqrt >= 0.75 * 0.75) {
    if (handInfo.selection != -1) {
      handInfo.selection = computeAngle(handInfo.stick);
    }
  } else {
    handInfo.selection = -1;
  }

  handInfo.clickingOld = handInfo.clicking;
  handInfo.clicking = controller.getTriggerStatus(hand);
}

void OVRController::update_status(AppStatus &status) const {
  vr::VRActiveActionSet_t action = {};
  action.ulActionSet = action_set_input;
  action.nPriority = vr::k_nActionSetOverlayGlobalPriorityMax;
  handle_input_err(vr::VRInput()->UpdateActionState(&action, sizeof(vr::VRActiveActionSet_t), 1));

  updateHand(*this, status, LeftRight::Left);
  updateHand(*this, status, LeftRight::Right);
}

void OVRController::set_texture(GLuint texture, LeftRight side) const {
  auto overlay_handle = overlay_handles[side];
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

bool OVRController::getTriggerStatus(LeftRight hand) const {
  vr::InputDigitalActionData_t digital_data = {};
  handle_input_err(vr::VRInput()->GetDigitalActionData(
      hand == LeftRight::Left ? action_left_click : action_right_click,
      &digital_data, sizeof(digital_data),
      vr::k_ulInvalidInputValueHandle));
  return digital_data.bState;
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
