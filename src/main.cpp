#include <iostream>
#include <SDL.h>
#include <GL/glew.h>
#include <oglwrap/oglwrap.h>

#include "OVRController.h"
#include "graphics/MainGuiRenderer.h"
#include "graphics/DesktopGuiRenderer.h"
#include "graphics/bmp_export.h"

#define WINDOW_CAPTION "clekeyOVR"
#define WINDOW_HEIGHT 1024
#define WINDOW_WIDTH 1024

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

  AppStatus status;
  status.chars = {
      u8"1", u8"2", u8"3", u8"4", u8"5", u8"6", u8"7", u8"8",
      u8"2", u8"a", u8"a", u8"a", u8"a", u8"a", u8"a", u8"a",
      u8"3", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042",
      u8"4", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044",
      u8"5", u8"C", u8"C", u8"C", u8"C", u8"C", u8"C", u8"C",
      u8"6", u8"c", u8"c", u8"c", u8"c", u8"c", u8"c", u8"c",
      u8"7", u8"D", u8"D", u8"D", u8"D", u8"D", u8"D", u8"D",
      u8"8", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=",
  };

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
    desktop_renderer->preDraw();

    main_renderer->draw(status, LeftRight::Left, true);
    ovr_controller.set_texture(main_renderer->dest_textures[LeftRight::Left].expose(), LeftRight::Left);

    main_renderer->draw(status, LeftRight::Right, false);
    ovr_controller.set_texture(main_renderer->dest_textures[LeftRight::Right].expose(), LeftRight::Right);

    //export_as_bmp(main_renderer.dest_texture, 0);

    desktop_renderer->drawTexture(main_renderer->dest_textures[LeftRight::Left], {-1, 0}, {1, 1});
    desktop_renderer->drawTexture(main_renderer->dest_textures[LeftRight::Right], {0, 0}, {1, 1});

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
