//
// Created by anatawa12 on 8/11/22.
//

#ifndef CLEKEY_OVR_MAINGUIRENDERER_H
#define CLEKEY_OVR_MAINGUIRENDERER_H

#include <GL/glew.h>
#include "oglwrap/oglwrap.h"
#include "BackgroundRingRenderer.h"
#include "CursorCircleRenderer.h"
#include "../OVRController.h"
#include "FreetypeRenderer.h"
#include "../AppStatus.h"

class MainGuiRenderer {
public:
  static std::unique_ptr<MainGuiRenderer> create(glm::ivec2 size);

  void drawRing(
      const KeyboardStatus &status,
      LeftRight side,
      bool alwaysShowInCircle,
      const RingOverlayConfig &config,
      gl::Texture2D &texture
  );

  void drawCenter(const KeyboardStatus &status, const CompletionOverlayConfig &config, gl::Texture2D &texture);

  glm::ivec2 size;

  gl::Renderbuffer depth_buffer;
  gl::Framebuffer frame_buffer;

  std::unique_ptr<BackgroundRingRenderer> backgroundRingRenderer;
  std::unique_ptr<CursorCircleRenderer> cursorCircleRenderer;
  std::unique_ptr<FreetypeRenderer> ftRenderer;
};

#endif //CLEKEY_OVR_MAINGUIRENDERER_H
