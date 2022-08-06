#include <iostream>
#include "openvr.h"

#ifdef WIN32
#include <windows.h>
#define sleep(n) Sleep(n * 1000)
#else
#include <csignal>
#define Sleep(n) usleep(n * 1000)
#endif

void handle_input_err(vr::EVRInputError error);

vr::VRActionHandle_t action_left_stick;
vr::VRActionHandle_t action_left_click;
vr::VRActionHandle_t action_left_haptic;
vr::VRActionHandle_t action_right_stick;
vr::VRActionHandle_t action_right_click;
vr::VRActionHandle_t action_right_haptic;
vr::VRActionSetHandle_t action_set_input;

int main() {
    std::cout << "Hello, World!" << std::endl;

    vr::HmdError err;
    vr::VR_Init(&err, vr::EVRApplicationType::VRApplication_Overlay);
    if (!vr::VROverlay()) {
        std::cerr << "error: " << vr::VR_GetVRInitErrorAsEnglishDescription(err) << std::endl;
        return -1;
    }
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

    std::cout << "action_left_stick:   " << action_left_stick << std::endl;
    std::cout << "action_left_click:   " << action_left_click << std::endl;
    std::cout << "action_left_haptic:  " << action_left_haptic << std::endl;
    std::cout << "action_right_stick:  " << action_right_stick << std::endl;
    std::cout << "action_right_click:  " << action_right_click << std::endl;
    std::cout << "action_right_haptic: " << action_right_haptic << std::endl;
    std::cout << "action_set_input:    " << action_set_input << std::endl;

    std::cout << "successfully launched" << std::endl;

    for (;;) {
        //*
        vr::VRActiveActionSet_t action = {};
        action.ulActionSet = action_set_input;
        handle_input_err(vr::VRInput()->UpdateActionState(&action, sizeof(vr::VRActiveActionSet_t), 1));
        vr::InputAnalogActionData_t analog_data = {};
        handle_input_err(vr::VRInput()->GetAnalogActionData(action_left_stick, &analog_data, sizeof (analog_data), vr::k_ulInvalidInputValueHandle));
        std::cout << "left input:  " << analog_data.bActive << ": " << analog_data.x << ", " << analog_data.y << std::endl;
        handle_input_err(vr::VRInput()->GetAnalogActionData(action_right_stick, &analog_data, sizeof (analog_data), vr::k_ulInvalidInputValueHandle));
        std::cout << "right input: " << analog_data.bActive << ": " << analog_data.x << ", " << analog_data.y << std::endl;
        vr::InputDigitalActionData_t digital_data = {};
        handle_input_err(vr::VRInput()->GetDigitalActionData(action_left_click, &digital_data, sizeof (digital_data), vr::k_ulInvalidInputValueHandle));
        std::cout << "right input: " << digital_data.bActive << ": " << digital_data.bState << std::endl;
        // */
        Sleep(100);
    }

    vr::VR_Shutdown();

    std::cout << "shutdown finished" << std::endl;

    return 0;
}

void handle_input_err(vr::EVRInputError error) {
    if (error != vr::VRInputError_None) {
        std::cerr << "input error: " << error << std::endl;
    }
}
