#include <iostream>
#include <SDL.h>
#include <GL/glew.h>
#include <oglwrap/oglwrap.h>

#include "OVRController.h"
#include "graphics/MainGuiRenderer.h"
#include "graphics/DesktopGuiRenderer.h"
#include "graphics/bmp_export.h"
#include "input_method/JapaneseInput.h"
#include "input_method/SignsInput.h"
#include "input_method/EnglishInput.h"

#define WINDOW_CAPTION "clekeyOVR"
#define WINDOW_HEIGHT 1024
#define WINDOW_WIDTH 1024

#ifdef WIN32
#include <Windows.h>
#include <stdio.h>
#endif

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
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);

  return window;
}

bool init_gl(SDL_Window *window) {
  SDL_GLContext context = SDL_GL_CreateContext(window);
  if (!context) {
    std::cerr << "SDL init error: " << SDL_GetError() << std::endl;
    return false;
  }

  glewExperimental = true;
  glewInit();

  gl::ClearColor(0.0f, 0.0f, 0.0f, 0.0f);

  return true;
}

int glmain(SDL_Window *window) {
  glm::ivec2 circleSize = {WINDOW_WIDTH, WINDOW_HEIGHT};
  auto main_renderer = MainGuiRenderer::create(circleSize);
  auto desktop_renderer = DesktopGuiRenderer::create(circleSize);
  OVRController ovr_controller;

  gl::Texture2D circleTextures[2];
  for (auto &dest_texture: circleTextures) {
    gl::Bind(dest_texture);
    dest_texture.upload(
        gl::kRgba8, WINDOW_WIDTH, WINDOW_HEIGHT,
        gl::kRgb, gl::kUnsignedByte, nullptr
    );
    dest_texture.magFilter(gl::kLinear);
    dest_texture.minFilter(gl::kLinear);
  }

  auto signInput = std::make_unique<SignsInput>();
  size_t index = 0;
  std::vector<std::unique_ptr<IInputMethod>> methods {};
  methods.emplace_back(std::move(std::make_unique<JapaneseInput>()));
  methods.emplace_back(std::move(std::make_unique<EnglishInput>()));
  IInputMethod *signInputPtr = signInput.get();

  AppStatus status {};
  status.method = methods[0].get();

  static const Uint32 interval = 1000 / 90;
  static Uint32 nextTime = SDL_GetTicks() + interval;

  for (;;) {
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

    ovr_controller.update_status(status);

    main_renderer->drawRing(status, LeftRight::Left, true, circleTextures[LeftRight::Left]);
    ovr_controller.set_texture(circleTextures[LeftRight::Left].expose(), LeftRight::Left);

    main_renderer->drawRing(status, LeftRight::Right, false, circleTextures[LeftRight::Right]);
    ovr_controller.set_texture(circleTextures[LeftRight::Right].expose(), LeftRight::Right);

    //export_as_bmp(main_renderer.dest_texture, 0);

    desktop_renderer->preDraw();
    desktop_renderer->drawTexture(circleTextures[LeftRight::Left], {-1, 0}, {1, 1});
    desktop_renderer->drawTexture(circleTextures[LeftRight::Right], {0, 0}, {1, 1});

    if ((status.left.clickStarted() || status.right.clickStarted())
        && status.left.selection != -1 && status.right.selection != -1) {
      auto action = status.method->onInput({status.left.selection, status.right.selection});
      switch (action) {
        case InputNextAction::Nop:
          // nop
          break;
        case InputNextAction::MoveToNextPlane:
          std::cout << "flush: " << (char *)status.method->getAndClearBuffer().c_str() << std::endl;
          if (++index == methods.size()) index = 0;
          status.method = methods[index].get();
          signInputPtr = signInput.get();
          break;
        case InputNextAction::MoveToSignPlane:
          std::cout << "flush: " << (char *)status.method->getAndClearBuffer().c_str() << std::endl;
          std::swap(signInputPtr, status.method);
          break;
        case InputNextAction::FlushBuffer:
          std::cout << "flush: " << (char *)status.method->getAndClearBuffer().c_str() << std::endl;
          break;
        case InputNextAction::RemoveLastChar:
          std::cout << "RemoveLastChar" << std::endl;
          break;
      }
    }

    SDL_GL_SwapWindow(window);

    int delayTime = (int) (nextTime - SDL_GetTicks());
    if (delayTime > 0) {
      SDL_Delay((Uint32) delayTime);
    }

    nextTime += interval;
  }
}

int main(int argc, char **argv) {
#ifdef WIN32
  SetConsoleOutputCP(CP_UTF8);
  setvbuf(stdout, nullptr, _IOFBF, 1024);
#endif
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
