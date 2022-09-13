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

bool init_ovr();

void shutdown_ovr();

class OVRController {
#ifdef WITH_OPEN_VR
  vr::VRActionHandle_t action_left_stick;
  vr::VRActionHandle_t action_left_click;
  vr::VRActionHandle_t action_left_haptic;
  vr::VRActionHandle_t action_right_stick;
  vr::VRActionHandle_t action_right_click;
  vr::VRActionHandle_t action_right_haptic;
  vr::VRActionSetHandle_t action_set_input;
  vr::VROverlayHandle_t overlay_handles[2];
#endif
public:
  OVRController();

  void update_status(AppStatus &) const;
  void set_texture(GLuint texture, LeftRight side) const;

  [[nodiscard]] glm::vec2 getStickPos(LeftRight hand) const;
  [[nodiscard]] bool getTriggerStatus(LeftRight right) const;

  OVRController(const OVRController&) = delete;
  OVRController& operator=(const OVRController&) = delete;
  OVRController(OVRController&&) = delete;
  OVRController& operator=(OVRController&&) = delete;

};


#endif //CLEKEY_OVR_OVRCONTROLLER_H
