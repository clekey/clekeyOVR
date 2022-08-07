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

#define WINDOW_CAPTION "clekeyOVR"
#define WINDOW_HEIGHT 256
#define WINDOW_WIDTH 512

void handle_input_err(vr::EVRInputError error);

vr::VRActionHandle_t action_left_stick;
vr::VRActionHandle_t action_left_click;
vr::VRActionHandle_t action_left_haptic;
vr::VRActionHandle_t action_right_stick;
vr::VRActionHandle_t action_right_click;
vr::VRActionHandle_t action_right_haptic;
vr::VRActionSetHandle_t action_set_input;

int main(int argc, char** argv) {
    if (SDL_Init(SDL_INIT_VIDEO)) {
        std::cerr << "sdl error: " << SDL_GetError() << std::endl;
        return -1;
    }

    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
    SDL_Window * window = SDL_CreateWindow(
            WINDOW_CAPTION,
            0, 0,
            WINDOW_WIDTH, WINDOW_HEIGHT,
            SDL_WINDOW_OPENGL);
    if (!window) {
        std::cerr << "sdl error: " << SDL_GetError() << std::endl;
        return -1;
    }

    SDL_GLContext context = SDL_GL_CreateContext(window);
    if (!context) return -1;

    // setup viewport
    glViewport(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
    glClearColor(0.0f, 0.0f, 0.0f, 0.0f);

    static const Uint32 interval = 1000 / 90;
    static Uint32 nextTime = SDL_GetTicks() + interval;
    while (true) {
        // check event

        SDL_Event ev;
        SDL_Keycode key;
        while ( SDL_PollEvent(&ev) )
        {
            switch(ev.type){
                case SDL_QUIT:
                    goto quit;
                case SDL_KEYDOWN:
                    key = ev.key.keysym.sym;
                    if(key == SDLK_ESCAPE)
                        goto quit;
                    break;
            }
        }

        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        // draw sphere
        glEnable(GL_COLOR_MATERIAL);
        glColor3ub(255, 0, 0);
        glBegin(GL_QUADS);
#define POLY_SIZE .5

        glVertex2d(-POLY_SIZE, -POLY_SIZE);
        glVertex2d(+POLY_SIZE, -POLY_SIZE);
        glVertex2d(+POLY_SIZE, +POLY_SIZE);
        glVertex2d(-POLY_SIZE, +POLY_SIZE);

        glVertex2d(-POLY_SIZE, -POLY_SIZE);
        glVertex2d(-POLY_SIZE, +POLY_SIZE);
        glVertex2d(+POLY_SIZE, +POLY_SIZE);
        glVertex2d(+POLY_SIZE, -POLY_SIZE);

        glEnd();

        SDL_GL_SwapWindow(window);

        int delayTime = (int) (nextTime - SDL_GetTicks());
        if (delayTime > 0) {
            SDL_Delay((Uint32) delayTime);
        }

        nextTime += interval;
    }
    quit:
    SDL_Quit();

#if 0
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
        std::cout << "left click:  " << digital_data.bActive << ": " << digital_data.bState << std::endl;
        handle_input_err(vr::VRInput()->GetDigitalActionData(action_right_click, &digital_data, sizeof (digital_data), vr::k_ulInvalidInputValueHandle));
        std::cout << "right click: " << digital_data.bActive << ": " << digital_data.bState << std::endl;
        // */
        Sleep(100);
    }

    vr::VR_Shutdown();

    std::cout << "shutdown finished" << std::endl;

    return 0;
#endif
}

void handle_input_err(vr::EVRInputError error) {
    if (error != vr::VRInputError_None) {
        std::cerr << "input error: " << error << std::endl;
    }
}
