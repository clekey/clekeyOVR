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

bool init_ovr();

void shutdown_ovr();

enum LeftRight {
  Left,
  Right,
};

class OVRController {
#ifdef WITH_OPEN_VR
  vr::VRActionHandle_t action_left_stick;
  vr::VRActionHandle_t action_left_click;
  vr::VRActionHandle_t action_left_haptic;
  vr::VRActionHandle_t action_right_stick;
  vr::VRActionHandle_t action_right_click;
  vr::VRActionHandle_t action_right_haptic;
  vr::VRActionSetHandle_t action_set_input;
  vr::VROverlayHandle_t overlay_handle;
#endif
public:
  OVRController();

  void input_tick() const;
  void tick(GLuint texture) const;

  [[nodiscard]] glm::vec2 getStickPos(LeftRight hand) const;

  OVRController(const OVRController&) = delete;
  OVRController& operator=(const OVRController&) = delete;
  OVRController(OVRController&&) = delete;
  OVRController& operator=(OVRController&&) = delete;
};


#endif //CLEKEY_OVR_OVRCONTROLLER_H
