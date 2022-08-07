#include <iostream>
#include "openvr.h"
#include <SDL.h>
#include <GL/glew.h>

#ifdef WIN32
#include <windows.h>
#define sleep(n) Sleep(n * 1000)
#else
#include <csignal>
#define Sleep(n) usleep(n * 1000)
#endif

void handle_input_err(vr::EVRInputError error);

const char *overlay_key = "com.anatawa12.clekey-ovr";

vr::VRActionHandle_t action_left_stick;
vr::VRActionHandle_t action_left_click;
vr::VRActionHandle_t action_left_haptic;
vr::VRActionHandle_t action_right_stick;
vr::VRActionHandle_t action_right_click;
vr::VRActionHandle_t action_right_haptic;
vr::VRActionSetHandle_t action_set_input;

int main(int argv, char** args) {
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

    vr::VROverlayHandle_t overlay_handle;
    vr::EVROverlayError overlay_err;
    if ((overlay_err = vr::VROverlay()->CreateOverlay(overlay_key, "clekey OVR", &overlay_handle))) {
        std::cerr << "error: " << vr::VROverlay()->GetOverlayErrorNameFromEnum(overlay_err) << std::endl;
    }

    vr::VROverlay()->SetOverlayWidthInMeters(overlay_handle, 2.0f);
    vr::VROverlay()->SetOverlayAlpha(overlay_handle, 0.5f);
    vr::VROverlay()->ShowOverlay(overlay_handle);

    //region sdl & GL initialization
    if ( SDL_Init( SDL_INIT_VIDEO | SDL_INIT_TIMER ) < 0 )
    {
        printf("%s - SDL could not initialize! SDL Error: %s\n", __FUNCTION__, SDL_GetError());
        return -1;
    }

    //SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
    //SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 1);
    //SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);

    SDL_GL_SetAttribute(SDL_GL_MULTISAMPLEBUFFERS, 0);
    SDL_GL_SetAttribute(SDL_GL_MULTISAMPLESAMPLES, 0);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_FLAGS, SDL_GL_CONTEXT_DEBUG_FLAG);

    SDL_Window *window = SDL_CreateWindow("clekey-vr", 320, 320, 320, 320, SDL_WINDOW_OPENGL | SDL_WINDOW_HIDDEN);
    SDL_GLContext context = SDL_GL_CreateContext(window);

    glewExperimental = GL_TRUE;
    GLenum nGlewError = glewInit();
    if (nGlewError != GLEW_OK)
    {
        printf("%s - Error initializing GLEW! %s\n", __FUNCTION__, glewGetErrorString(nGlewError));
        return -1;
    }
    glGetError(); // to clear the error caused deep in GLEW
    if ( SDL_GL_SetSwapInterval(0) < 0 ){
        printf("%s - Warning: Unable to set VSync! SDL Error: %s\n", __FUNCTION__, SDL_GetError() );
        return -1;
    }

    GLuint tex;
    glGenTextures(1, &tex);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAX_LEVEL, 0);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, 320, 320, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, tex, 0);

    //endregion

    std::cout << "successfully launched" << std::endl;

    for (;;) {
        if (vr::VROverlay()->IsOverlayVisible(overlay_handle)) {
            vr::HmdMatrix34_t pose = {
                    {
                            {1, 0, 0, 0},
                            {0, 1, 0, 0},
                            {0, 0, 1, -10},
                    }
            };
            vr::VROverlay()->SetOverlayTransformTrackedDeviceRelative(overlay_handle, 0, &pose);
            glClear(GL_COLOR_BUFFER_BIT);

            vr::Texture_t vr_tex = {(void *) (uintptr_t) tex, vr::TextureType_OpenGL, vr::ColorSpace_Auto};
            vr::VROverlay()->SetOverlayTexture(overlay_handle, &vr_tex);

        }

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
