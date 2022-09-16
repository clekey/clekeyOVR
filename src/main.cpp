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

void copyClipboard(const std::u8string &);

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

class KeyboardManager {
  std::unique_ptr<IInputMethod> signInput;
  size_t index;
  std::vector<std::unique_ptr<IInputMethod>> methods;
  IInputMethod *signInputPtr;
public:
  KeyboardStatus status;

  KeyboardManager();

  void flush() const;

  bool tick();

  void swapSignInput() {
    std::swap(signInputPtr, status.method);
  }

  void moveToNextKeyboard() {
    if (++index == methods.size()) index = 0;
    signInputPtr = signInput.get();
    status.method = methods[index].get();
  }
};

KeyboardManager::KeyboardManager() :
    signInput(std::make_unique<SignsInput>()),
    index(0),
    methods{},
    signInputPtr(signInput.get()) {
  methods.emplace_back(std::make_unique<JapaneseInput>());
  methods.emplace_back(std::make_unique<EnglishInput>());
  status.method = methods[index].get();
}

class Application {
  std::unique_ptr<MainGuiRenderer> main_renderer;
  std::unique_ptr<DesktopGuiRenderer> desktop_renderer;
  OVRController ovr_controller;
  gl::Texture2D circleTextures[2];
  gl::Texture2D centerTexture;
  KeyboardManager keyboard;
  AppStatus status;

  bool SDLTick();

  void waitingTick();

  void inputtingTick();

  void suspendingTick();

public:
  Application();

  bool tick();
};

gl::Texture2D makeTexture(GLsizei width, GLsizei height) {
  gl::Texture2D centerTexture;
  gl::Bind(centerTexture);
  centerTexture.upload(
      gl::kRgba8, width, height,
      gl::kRgb, gl::kUnsignedByte, nullptr
  );
  centerTexture.magFilter(gl::kLinear);
  centerTexture.minFilter(gl::kLinear);
  return std::move(centerTexture);
}

Application::Application() :
    main_renderer(MainGuiRenderer::create({WINDOW_WIDTH, WINDOW_HEIGHT})),
    desktop_renderer(DesktopGuiRenderer::create({WINDOW_WIDTH, WINDOW_HEIGHT})),
    ovr_controller(),
    circleTextures{makeTexture(WINDOW_WIDTH, WINDOW_HEIGHT), makeTexture(WINDOW_WIDTH, WINDOW_HEIGHT)},
    centerTexture(makeTexture(WINDOW_WIDTH, WINDOW_HEIGHT / 8)),
    status(AppStatus::Waiting) {}

bool Application::tick() {
  if (SDLTick()) return true;
  switch (status) {
    case AppStatus::Waiting:
      waitingTick();
      break;
    case AppStatus::Inputting:
      inputtingTick();
      break;
    case AppStatus::Suspending:
      suspendingTick();
      break;
  }
  return false;
}

bool Application::SDLTick() {
  SDL_Event ev;
  SDL_Keycode key;
  while (SDL_PollEvent(&ev)) {
    switch (ev.type) {
      case SDL_QUIT:
        return true;
      case SDL_KEYDOWN:
        key = ev.key.keysym.sym;
        if (key == SDLK_ESCAPE)
          return true;
        break;
    }
  }
  return false;
}

void Application::waitingTick() {
  ovr_controller.setActiveActionSet({ActionSetKind::Waiting});
  ovr_controller.hideOverlays();
  if (ovr_controller.getButtonStatus(ButtonKind::BeginInput))
    status = AppStatus::Inputting;
}

void Application::inputtingTick() {
  ovr_controller.setActiveActionSet({ActionSetKind::Suspender, ActionSetKind::Input});
  ovr_controller.update_status(keyboard.status);

  main_renderer->drawRing(keyboard.status, LeftRight::Left, true, circleTextures[LeftRight::Left]);
  ovr_controller.set_texture(circleTextures[LeftRight::Left].expose(), LeftRight::Left);

  main_renderer->drawRing(keyboard.status, LeftRight::Right, false, circleTextures[LeftRight::Right]);
  ovr_controller.set_texture(circleTextures[LeftRight::Right].expose(), LeftRight::Right);

  if (keyboard.status.method->getBuffer().length()) {
    main_renderer->drawCenter(keyboard.status, centerTexture);
    ovr_controller.setCenterTexture(centerTexture.expose());
  } else {
    ovr_controller.closeCenterOverlay();
  }

  //export_as_bmp(main_renderer.dest_texture, 0);

  desktop_renderer->preDraw();
  desktop_renderer->drawTexture(circleTextures[LeftRight::Left], {-1, 0}, {1, 1});
  desktop_renderer->drawTexture(circleTextures[LeftRight::Right], {0, 0}, {1, 1});
  desktop_renderer->drawTexture(centerTexture, {-1, -.25}, {2, .25});

  if (keyboard.tick()) {
    status = AppStatus::Waiting;
  } else if (ovr_controller.getButtonStatus(ButtonKind::SuspendInput)) {
    status = AppStatus::Suspending;
  }
}

void Application::suspendingTick() {
  ovr_controller.setActiveActionSet({ActionSetKind::Suspender});
  ovr_controller.hideOverlays();
  if (!ovr_controller.getButtonStatus(ButtonKind::SuspendInput))
    status = AppStatus::Inputting;
}

void KeyboardManager::flush() const {
  auto buffer = status.method->getAndClearBuffer();
  if (buffer.empty()) return;
  std::cout << "flush: " << (char *) buffer.c_str() << std::endl;
  copyClipboard(buffer);
}

bool KeyboardManager::tick() {
  if ((status.left.clickStarted() || status.right.clickStarted())
      && status.left.selection != -1 && status.right.selection != -1) {
    auto action = status.method->onInput({status.left.selection, status.right.selection});
    switch (action) {
      case InputNextAction::Nop:
        // nop
        break;
      case InputNextAction::MoveToNextPlane:
        flush();
        moveToNextKeyboard();
        break;
      case InputNextAction::MoveToSignPlane:
        flush();
        swapSignInput();
        break;
      case InputNextAction::FlushBuffer:
        flush();
        break;
      case InputNextAction::RemoveLastChar:
#ifdef WIN32
        std::cout << "simulate backspace" << std::endl;
        keybd_event(VK_BACK, 0, 0, 0);
        keybd_event(VK_BACK, 0, KEYEVENTF_KEYUP, 0);
#endif
        std::cout << "RemoveLastChar" << std::endl;
        break;
      case InputNextAction::CloseKeyboard:
        flush();
        return true; // TODO: just close overlay and wait for request to open
      case InputNextAction::NewLine:
        flush();
#ifdef WIN32
        std::cout << "simulate return" << std::endl;
        keybd_event(VK_RETURN, 0, 0, 0);
        keybd_event(VK_RETURN, 0, KEYEVENTF_KEYUP, 0);
#endif
        std::cout << "RemoveLastChar" << std::endl;
        break;
    }
  }
  return false;
}

int glmain(SDL_Window *window) {
  Application application;

  static const Uint32 interval = 1000 / 90;
  static Uint32 nextTime = SDL_GetTicks() + interval;

  for (;;) {
    if (application.tick())
      return 0;

    SDL_GL_SwapWindow(window);

    int delayTime = (int) (nextTime - SDL_GetTicks());
    if (delayTime > 0) {
      SDL_Delay((Uint32) delayTime);
    }

    nextTime += interval;
  }
}

void copyClipboard(const std::u8string &buffer) {
#if defined(WIN32)
  if (buffer.length() == 1) {
    char c = char(buffer[0]);
    if ('0' <= c && c <= '9') {
      keybd_event(c, 0, 0, 0);
      keybd_event(c, 0, KEYEVENTF_KEYUP, 0);
      return;
    } else if ('A' <= c && c <= 'Z') {
      keybd_event(VK_LSHIFT, 0, 0, 0);
      keybd_event(c, 0, 0, 0);
      keybd_event(c, 0, KEYEVENTF_KEYUP, 0);
      keybd_event(VK_LSHIFT, 0, KEYEVENTF_KEYUP, 0);
      return;
    } else if ('a' <= c && c <= 'z') {
      c += ('A' - 'a');
      keybd_event(c, 0, 0, 0);
      keybd_event(c, 0, KEYEVENTF_KEYUP, 0);
      return;
    }
  }
  if (!OpenClipboard(NULL)) {
    std::cout << "Cannot open the Clipboard" << std::endl;
    return;
  }
  // Remove the current Clipboard contents
  if (!EmptyClipboard()) {
    std::cout << "Cannot empty the Clipboard" << std::endl;
    return;
  }
  // create in utf16 string
  std::u16string u16buffer;
  u16buffer.reserve(buffer.length());

  for (char32_t u32char: make_u8u32range(buffer)) {
    if (u32char <= 0xFFFF) {
      u16buffer.push_back(char16_t(u32char));
    } else {
      uint32_t index = u32char - 0x10000;
      char16_t hsg = (index >> 10) + 0xD800;
      char16_t lsg = (index & 0x3FF) + 0xDC00;
      u16buffer.push_back(hsg);
      u16buffer.push_back(lsg);
    }
  }

  // Get the currently selected data
  size_t u16Size((u16buffer.length() + 1) * 2);
  HGLOBAL hGlob = GlobalAlloc(GMEM_FIXED, u16Size);
  memcpy((void *) hGlob, (void *) u16buffer.c_str(), u16Size);
  // For the appropriate data formats...
  if (SetClipboardData(CF_UNICODETEXT, hGlob) == NULL) {
    std::cout << "Unable to set Clipboard data, error: " << GetLastError() << std::endl;
    CloseClipboard();
    GlobalFree(hGlob);
    return;
  }
  CloseClipboard();

  keybd_event(VK_LCONTROL, 0, 0, 0);
  keybd_event('V', 0, 0, 0);
  keybd_event('V', 0, KEYEVENTF_KEYUP, 0);
  keybd_event(VK_LCONTROL, 0, KEYEVENTF_KEYUP, 0);
#endif
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
