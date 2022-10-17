#include <iostream>
#include <SDL.h>

#include "OVRController.h"
#include "graphics/MainGuiRenderer.h"
#include "input_method/JapaneseInput.h"
#include "input_method/SignsInput.h"
#include "input_method/EnglishInput.h"
#include "Config.h"

//// skia
#include <include/gpu/GrBackendSurface.h>
#include <include/gpu/GrDirectContext.h>
#include <include/gpu/gl/GrGLInterface.h>
#include <include/core/SkCanvas.h>
#include <include/core/SkColorSpace.h>
#include <include/core/SkSurface.h>
#include <include/core/SkFont.h>
#include <src/gpu/ganesh/gl/GrGLDefines_impl.h>
////
#include "opengl.h"
//#include "graphics/bmp_export.h"
#include "graphics/glutil.h"
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

  Uint32 windowFlags = SDL_WINDOW_OPENGL;
#ifdef NDEBUG
  windowFlags |= SDL_WINDOW_HIDDEN;
#endif
  SDL_Window *window = SDL_CreateWindow(
      WINDOW_CAPTION,
      0, 0,
      WINDOW_WIDTH, WINDOW_HEIGHT,
      windowFlags);
  if (!window) {
    std::cerr << "sdl error: " << SDL_GetError() << std::endl;
    return nullptr;
  }

  SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 1);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_FLAGS, SDL_GL_CONTEXT_DEBUG_FLAG);
  SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 8);
  SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 8);
  SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 8);
  SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 0);
  SDL_GL_SetAttribute(SDL_GL_STENCIL_SIZE, 8);
  SDL_GL_SetAttribute(SDL_GL_ACCELERATED_VISUAL, 1);

  return window;
}

GrDirectContext* grContext;
#ifndef NODEBUG
sk_sp<SkSurface> windowSurface;
#endif

bool init_gl(SDL_Window *window) {
  SDL_GLContext context = SDL_GL_CreateContext(window);
  if (!context) {
    std::cerr << "SDL init error: " << SDL_GetError() << std::endl;
    return false;
  }

  grContext = GrDirectContext::MakeGL().release();
  if (!grContext) {
    std::cerr << "GrContext creation failed" << std::endl;
    return false;
  }

#ifndef NODEBUG
  int success =  SDL_GL_MakeCurrent(window, context);
  if (success != 0) {
    std::cerr << "SDL GL make current failed: " << SDL_GetError() << std::endl;
    return false;
  }

  // init opengl for window
  glViewport(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
  glClearColor(1, 1, 1, 1);
  glClearStencil(0);
  glClear(GL_COLOR_BUFFER_BIT | GL_STENCIL_BUFFER_BIT);
  GrGLFramebufferInfo bufferInfo;
  glGetIntegerv(GR_GL_FRAMEBUFFER_BINDING, (GLint *)&bufferInfo.fFBOID);
  bufferInfo.fFormat = GL_RGBA8;
  auto target = GrBackendRenderTarget(WINDOW_WIDTH, WINDOW_HEIGHT, 0, 8, bufferInfo);
  windowSurface = SkSurface::MakeFromBackendRenderTarget(
      grContext, target,
      kBottomLeft_GrSurfaceOrigin, kRGBA_8888_SkColorType,
      nullptr, nullptr);
#endif

  return true;
}

class KeyboardManager {
  OVRController *ovr_controller;
  std::unique_ptr<IInputMethod> signInput;
  size_t index;
  std::vector<std::unique_ptr<IInputMethod>> methods;
  IInputMethod *signInputPtr;
public:
  KeyboardStatus status;

  KeyboardManager(OVRController *ovr_controller);

  void flush() const;

  bool tick();

  bool doInputAction(InputNextAction);

  void swapSignInput() {
    std::swap(signInputPtr, status.method);
  }

  void moveToNextKeyboard() {
    if (++index == methods.size()) index = 0;
    signInputPtr = signInput.get();
    status.method = methods[index].get();
  }
};

KeyboardManager::KeyboardManager(OVRController *ovr_controller) :
    ovr_controller(ovr_controller),
    signInput(std::make_unique<SignsInput>()),
    index(0),
    methods{},
    signInputPtr(signInput.get()) {
  methods.emplace_back(std::make_unique<JapaneseInput>());
  methods.emplace_back(std::make_unique<EnglishInput>());
  status.method = methods[index].get();
}

struct GlTextureSurface {
  GLuint glId;
  sk_sp<SkSurface> surface;
  sk_sp<SkImage> image;
};

class Application {
  CleKeyConfig config;
  std::unique_ptr<MainGuiRenderer> main_renderer;
  std::unique_ptr<OVRController> ovr_controller;
  GlTextureSurface circleTextures[2];
  GlTextureSurface centerTexture;
  KeyboardManager keyboard;
  AppStatus status;

  void setStatus(AppStatus status);

  bool SDLTick();

  void waitingTick();

  void inputtingTick();

  void suspendingTick();

public:
  Application();

  bool tick();
};

GlTextureSurface makeSurface(GLsizei width, GLsizei height) {
  GrGLTextureInfo texInfo;
  texInfo.fTarget = GL_TEXTURE_2D;
  glGenTextures(1, &texInfo.fID);
  texInfo.fFormat = GL_RGBA8;
  glBindTexture(GL_TEXTURE_2D, texInfo.fID);
  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, width, height, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  check_gl_err("create surface texture");

  GrBackendTexture backendTexture(width, height, GrMipmapped::kNo, texInfo);
  auto surface = SkSurface::MakeFromBackendTexture(
      grContext, backendTexture,
      kBottomLeft_GrSurfaceOrigin, 0,
      kRGBA_8888_SkColorType, nullptr, nullptr
  );
  auto image = SkImage::MakeFromTexture(
      grContext, backendTexture,
      kBottomLeft_GrSurfaceOrigin, kRGBA_8888_SkColorType,
      kOpaque_SkAlphaType, nullptr);

  if (!surface)
    abort();

  return GlTextureSurface {texInfo.fID, surface, image};
}

Application::Application() :
    config(),
    main_renderer(MainGuiRenderer::create({WINDOW_WIDTH, WINDOW_HEIGHT})),
    ovr_controller(new OVRController()),
    circleTextures{makeSurface(WINDOW_WIDTH, WINDOW_HEIGHT), makeSurface(WINDOW_WIDTH, WINDOW_HEIGHT)},
    centerTexture(makeSurface(WINDOW_WIDTH, WINDOW_HEIGHT / 8)),
    keyboard(ovr_controller.get()),
#if defined(WITH_OPEN_VR)
    status(AppStatus::Waiting)
#else
    status(AppStatus::Inputting)
#endif
{
  loadConfig(config);
  ovr_controller->loadConfig(config);
  std::cout << "left tex id:   " << circleTextures[0].glId << std::endl;
  std::cout << "right tex id:  " << circleTextures[1].glId << std::endl;
  std::cout << "center tex id: " << centerTexture.glId << std::endl;
}

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
  ovr_controller->setActiveActionSet({ActionSetKind::Waiting});
  ovr_controller->hideOverlays();
  if (ovr_controller->isClickStarted(HardKeyButton::CloseButton))
    setStatus(AppStatus::Inputting);
}

void Application::inputtingTick() {
  ovr_controller->setActiveActionSet({ActionSetKind::Suspender, ActionSetKind::Input, ActionSetKind::Waiting});
  ovr_controller->update_status(keyboard.status);

  main_renderer->drawRing(keyboard.status, LeftRight::Left, true, config.leftRing, *circleTextures[LeftRight::Left].surface);
  main_renderer->drawRing(keyboard.status, LeftRight::Right, false, config.rightRing, *circleTextures[LeftRight::Right].surface);

  if (keyboard.status.method->getBuffer().length()) {
    main_renderer->drawCenter(keyboard.status, config.completion, *centerTexture.surface);
  }

  check_gl_err("inputtingTick; after flush&submit");

  ovr_controller->set_texture(circleTextures[LeftRight::Left].glId, LeftRight::Left);
  ovr_controller->set_texture(circleTextures[LeftRight::Right].glId, LeftRight::Right);
  if (keyboard.status.method->getBuffer().length()) {
    ovr_controller->setCenterTexture(centerTexture.glId);
  } else {
    ovr_controller->closeCenterOverlay();
  }

  check_gl_err("inputtingTick; after set texture");

  //export_as_bmp(circleTextures[LeftRight::Left].glId, 0);
  //export_as_bmp(circleTextures[LeftRight::Right].glId, 0);

#ifndef NDEBUG
  // draw debug desktop
  {
    SkCanvas *canvas = windowSurface->getCanvas();
    canvas->clear(SK_ColorTRANSPARENT);
    canvas->drawImageRect(
        circleTextures[LeftRight::Left].image,
        SkRect::MakeXYWH(0, 0, WINDOW_WIDTH / 2.0f, WINDOW_HEIGHT / 2.0f),
        SkSamplingOptions());

    canvas->drawImageRect(
        circleTextures[LeftRight::Right].image,
        SkRect::MakeXYWH(WINDOW_WIDTH / 2.0f, 0, WINDOW_WIDTH / 2.0f, WINDOW_HEIGHT / 2.0f),
        SkSamplingOptions());

    canvas->drawImageRect(
        centerTexture.image,
        SkRect::MakeXYWH(0, WINDOW_HEIGHT / 2.0f, WINDOW_WIDTH, WINDOW_HEIGHT / 8.0f),
        SkSamplingOptions());
  }
#endif

  if (keyboard.tick()) {
    setStatus(AppStatus::Waiting);
  } else if (ovr_controller->getButtonStatus(ButtonKind::SuspendInput)) {
    setStatus(AppStatus::Suspending);
  }
}

void Application::suspendingTick() {
  ovr_controller->setActiveActionSet({ActionSetKind::Suspender});
  ovr_controller->hideOverlays();
  if (!ovr_controller->getButtonStatus(ButtonKind::SuspendInput))
    setStatus(AppStatus::Inputting);
}

void Application::setStatus(AppStatus value) {
  status = value;
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
    if (doInputAction(status.method->onInput({status.left.selection, status.right.selection})))
      return true;
  }
  for (const auto item: HardKeyButtonValues)
    if (ovr_controller->isClickStarted(item))
      if (doInputAction(status.method->onHardInput(item)))
        return true;
  return false;
}

bool KeyboardManager::doInputAction(InputNextAction action) {
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
  return false;
}

int glmain(SDL_Window *window) {
  Application application;

  static const Uint32 interval = 1000 / 30;
  static Uint32 nextTime = SDL_GetTicks() + interval;

  for (;;) {
#ifndef NDEBUG
    windowSurface->getCanvas()->clear(SK_ColorTRANSPARENT);
#endif
    if (application.tick())
      return 0;

    grContext->flush();
    grContext->submit();

#ifndef NDEBUG
    SDL_GL_SwapWindow(window);
#endif

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
