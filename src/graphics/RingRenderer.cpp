//
// Created by anatawa12 on 2022/09/11.
//

#include "RingRenderer.h"

namespace {

constexpr float sin45deg = 0.70710678118655;

inline std::array<glm::vec2, 8> calcOffsets(float size) {
  float axis = 0.375f * size;
  float diagonal = axis * sin45deg;
  return {
      glm::vec2{.0f, +axis},
      glm::vec2{+diagonal, +diagonal},
      glm::vec2{+axis, .0f},
      glm::vec2{+diagonal, -diagonal},
      glm::vec2{.0f, -axis},
      glm::vec2{-diagonal, -diagonal},
      glm::vec2{-axis, .0f},
      glm::vec2{-diagonal, +diagonal},
  };
}

void renderRingChars(FreetypeRenderer &renderer, glm::vec2 center, glm::vec3 color, float size,
                     auto getChar) {
  float fontSize = size * 0.2f;
  auto offsets = calcOffsets(size);

  for (int i = 0; i < 8; ++i) {
    renderer.addCenteredStringWithMaxWidth(getChar(i), center + offsets[i], color, fontSize, fontSize,
                                           CenteredMode::Both);
  }
}

}

void RingRenderer::render(
    glm::vec2 center,
    glm::vec2 stickPos,
    float size,
    RingDirection direction,
    int selectingCurrent,
    int selectingOther,
    std::array<std::u8string, 64> chars
) {
  brRenderer.draw(center, {size, size}, centerColor, backgroundColor, edgeColor);
  int lineStep = direction == RingDirection::Horizontal ? 1 : 8;
  int lineLen = direction == RingDirection::Horizontal ? 8 : 1;

  if (selectingOther == -1) {
    auto offsets = calcOffsets(size);
    float innerSize = size * 0.2f;
    for (int pos = 0; pos < 8; ++pos) {
      int colOrigin = lineStep * pos;
      renderRingChars(ftRenderer, center + offsets[pos], charColor, innerSize, [&chars, lineLen, colOrigin](int idx) -> std::u8string& {
        return chars[colOrigin + lineLen * idx];
      });
    }
  } else {
    int lineOrigin = lineLen * selectingOther;
    renderRingChars(ftRenderer, center, charColor, size, [&chars, lineStep, lineOrigin](int idx) -> std::u8string& {
      return chars[lineOrigin + lineStep * idx];
    });
  }
  ftRenderer.doDraw();

  ccRenderer.draw(center, {size, size}, stickPos);
}

RingRenderer::RingRenderer(
    FreetypeRenderer &ftRenderer,
    CursorCircleRenderer &ccRenderer,
    BackgroundRingRenderer &brRenderer,
    const glm::vec3 &charColor,
    const glm::vec3 &selectingCharColor,
    const glm::vec4 &centerColor,
    const glm::vec4 &backgroundColor,
    const glm::vec4 &edgeColor
) : ftRenderer(ftRenderer),
    ccRenderer(ccRenderer),
    brRenderer(brRenderer),
    centerColor(centerColor),
    backgroundColor(backgroundColor),
    edgeColor(edgeColor),
    charColor(charColor),
    selectingCharColor(selectingCharColor) {}