//
// Created by anatawa12 on 8/11/22.
//

#ifndef CLEKEY_OVR_OVRCONTROLLER_H
#define CLEKEY_OVR_OVRCONTROLLER_H

#include "openvr.h"

bool init_ovr();
void shutdown_ovr();

class OVRController {
    vr::VRActionHandle_t action_left_stick;
    vr::VRActionHandle_t action_left_click;
    vr::VRActionHandle_t action_left_haptic;
    vr::VRActionHandle_t action_right_stick;
    vr::VRActionHandle_t action_right_click;
    vr::VRActionHandle_t action_right_haptic;
    vr::VRActionSetHandle_t action_set_input;
public:
    OVRController();
    void tick() const;
};


#endif //CLEKEY_OVR_OVRCONTROLLER_H
