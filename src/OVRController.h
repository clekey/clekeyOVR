//
// Created by anatawa12 on 8/11/22.
//

#ifndef CLEKEY_OVR_OVRCONTROLLER_H
#define CLEKEY_OVR_OVRCONTROLLER_H

#ifdef WITH_OPEN_VR

#include "openvr.h"

#endif

#include "GL/glew.h"

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
  vr::VROverlayHandle_t overlay_handle;
#endif
public:
  OVRController();

  void tick(GLuint texture) const;
};


#endif //CLEKEY_OVR_OVRCONTROLLER_H
