#include <iostream>
#include <SDL.h>
#include <GL/glew.h>
#include <oglwrap/oglwrap.h>

#include "graphics/OVRController.h"
#include "graphics/MainGuiRenderer.h"
#include "graphics/DesktopGuiRenderer.h"
#include "graphics/bmp_export.h"

#define WINDOW_CAPTION "clekeyOVR"
#define WINDOW_HEIGHT 256
#define WINDOW_WIDTH 512

// error handling

SDL_Window *init_SDL() {
    if (SDL_Init(SDL_INIT_VIDEO)) {
        std::cerr << "sdl error: " << SDL_GetError() << std::endl;
        return nullptr;
    }

    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
    SDL_Window *window = SDL_CreateWindow(
            WINDOW_CAPTION,
            0, 0,
            WINDOW_WIDTH, WINDOW_HEIGHT,
            SDL_WINDOW_OPENGL);
    if (!window) {
        std::cerr << "sdl error: " << SDL_GetError() << std::endl;
        return nullptr;
    }

    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 1);

    return window;
}

bool init_gl(SDL_Window *window) {
    SDL_GLContext context = SDL_GL_CreateContext(window);
    if (!context) return false;

    glewExperimental = true;
    glewInit();

    gl::ClearColor(0.0f, 0.0f, 0.0f, 0.0f);

    return true;
}

int glmain(SDL_Window *window) {
    MainGuiRenderer main_renderer(WINDOW_WIDTH, WINDOW_HEIGHT);
    DesktopGuiRenderer desktop_renderer(WINDOW_WIDTH, WINDOW_HEIGHT);
    OVRController ovr_controller;

    static const Uint32 interval = 1000 / 90;
    static Uint32 nextTime = SDL_GetTicks() + interval;

    for (;;) {
        //*
        ovr_controller.tick();

        SDL_Event ev;
        SDL_Keycode key;
        while (SDL_PollEvent(&ev)) {
            switch (ev.type) {
                case SDL_QUIT:
                    return 0;
                case SDL_KEYDOWN:
                    key = ev.key.keysym.sym;
                    if (key == SDLK_ESCAPE)
                        return 0;
                    break;
            }
        }

        main_renderer.draw();

        export_as_bmp(main_renderer.rendered_textures[0].texture, 0);

        desktop_renderer.draw(main_renderer.rendered_textures[0].texture);

        SDL_GL_SwapWindow(window);

        int delayTime = (int) (nextTime - SDL_GetTicks());
        if (delayTime > 0) {
            SDL_Delay((Uint32) delayTime);
        }

        nextTime += interval;
    }
}

int main(int argc, char **argv) {
    SDL_Window *window = init_SDL();
    if (!window) return 1;
    if (!init_gl(window)) return 2;
    if (!init_ovr()) return 3;

    int exit_code = glmain(window);

    SDL_Quit();
    shutdown_ovr();

    std::cout << "shutdown finished" << std::endl;

    return exit_code;
}
