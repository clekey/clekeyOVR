//
// Created by anatawa12 on 2022/09/03.
//

#ifndef CLEKEY_OVR_CURSORCIRCLERENDERER_H
#define CLEKEY_OVR_CURSORCIRCLERENDERER_H


#include <include/core/SkCanvas.h>
#include <memory>
#include <glm/glm.hpp>

class CursorCircleRenderer {
public:
  static std::unique_ptr<CursorCircleRenderer> create();

  void draw(
      SkCanvas *canvas,
      SkPoint center,
      float radius,
      glm::vec2 stick,
      glm::vec4 color = glm::vec4(0.22, 0.22, 0.22, 1.0)
  );
};


#endif //CLEKEY_OVR_CURSORCIRCLERENDERER_H
