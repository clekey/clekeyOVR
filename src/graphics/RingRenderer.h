//
// Created by anatawa12 on 2022/09/11.
//

#ifndef CLEKEY_OVR_RINGRENDERER_H
#define CLEKEY_OVR_RINGRENDERER_H

#include "FreetypeRenderer.h"
#include "CursorCircleRenderer.h"
#include "BackgroundRingRenderer.h"
#include <array>

enum class RingDirection {
  Horizontal,
  Vertical,
};

class RingRenderer {
  FreetypeRenderer &ftRenderer;
  CursorCircleRenderer &ccRenderer;
  BackgroundRingRenderer &brRenderer;

  glm::vec4 centerColor;
  glm::vec4 backgroundColor;
  glm::vec4 edgeColor;
  glm::vec3 charColor;
  glm::vec3 selectingCharColor;

public:
  RingRenderer(
      FreetypeRenderer &ftRenderer,
      CursorCircleRenderer &ccRenderer,
      BackgroundRingRenderer &brRenderer,
      const glm::vec3 &charColor,
      const glm::vec3 &selectingCharColor,
      const glm::vec4 &centerColor = glm::vec4(0.83, 0.83, 0.83, 1.0),
      const glm::vec4 &backgroundColor = glm::vec4(0.686, 0.686, 0.686, 1.0),
      const glm::vec4 &edgeColor = glm::vec4(1.0, 1.0, 1.0, 1.0)
  );

  void render(
      glm::vec2 center,
      glm::vec2 stickPos,
      float size,
      RingDirection direction,
      int selectingCurrent,
      int selectingOther,
      std::array<std::u8string, 64> chars
  );
};

#endif //CLEKEY_OVR_RINGRENDERER_H
