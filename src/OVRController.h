//
// Created by anatawa12 on 8/11/22.
//

#ifndef CLEKEY_OVR_OVRCONTROLLER_H
#define CLEKEY_OVR_OVRCONTROLLER_H

#ifdef WITH_OPEN_VR

#include "openvr.h"

#endif

#include "GL/glew.h"
#include "glm/vec2.hpp"
#include "AppStatus.h"
#include <vector>

bool init_ovr();

void shutdown_ovr();

enum class ActionSetKind {
  Input,
  Waiting,
  Suspender
};

enum class ButtonKind {
  BeginInput,
  SuspendInput,
};

class OVRController {
#ifdef WITH_OPEN_VR
  // input
  vr::VRActionHandle_t action_input_left_stick;
  vr::VRActionHandle_t action_input_left_click;
  vr::VRActionHandle_t action_input_left_haptic;
  vr::VRActionHandle_t action_input_right_stick;
  vr::VRActionHandle_t action_input_right_click;
  vr::VRActionHandle_t action_input_right_haptic;
  vr::VRActionSetHandle_t action_set_input;

  // waiting
  vr::VRActionHandle_t action_waiting_begin_input;
  vr::VRActionSetHandle_t action_set_waiting;

  // suspender
  vr::VRActionHandle_t action_suspender_suspender;
  vr::VRActionSetHandle_t action_set_suspender;

  vr::VROverlayHandle_t overlay_handles[3];
#endif
public:
  OVRController();

  void setActiveActionSet(std::vector<ActionSetKind> kinds) const;
  void update_status(KeyboardStatus &) const;
  void set_texture(GLuint texture, LeftRight side) const;
  void setCenterTexture(GLuint texture) const;
  void closeCenterOverlay() const;

  [[nodiscard]] glm::vec2 getStickPos(LeftRight hand) const;
  [[nodiscard]] bool getTriggerStatus(LeftRight right) const;
  void playHaptics(
      LeftRight hand,
      float fStartSecondsFromNow,
      float fDurationSeconds,
      float fFrequency,
      float fAmplitude
  ) const;
  [[nodiscard]] bool getButtonStatus(ButtonKind kind) const;

  OVRController(const OVRController&) = delete;
  OVRController& operator=(const OVRController&) = delete;
  OVRController(OVRController&&) = delete;
  OVRController& operator=(OVRController&&) = delete;

  void hideOverlays();
};


#endif //CLEKEY_OVR_OVRCONTROLLER_H
