//
// Created by anatawa12 on 2022/09/11.
//

#include "RingRenderer.h"

namespace {

constexpr float sin45deg = 0.70710678118655;

inline std::array<glm::vec2, 8> calcOffsets(float size) {
  float axis = 0.75f * size;
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

void renderRingChars(FreetypeRenderer &renderer, glm::vec2 center, float size,
                     std::function<std::pair<std::u8string&, glm::vec3>(int)> getChar) {
  float fontSize = size * 0.4f;
  auto offsets = calcOffsets(size);

  for (int i = 0; i < 8; ++i) {
    auto pair = getChar(i);
    renderer.addCenteredStringWithMaxWidth(pair.first, center + offsets[i], pair.second, fontSize, fontSize,
                                           CenteredMode::Both);
  }
}

}

void
RingRenderer::render(glm::vec2 stickPos, RingDirection direction, int selectingCurrent, int selectingOther,
                     std::array<std::u8string, 64> chars) {
  brRenderer.draw(centerColor, backgroundColor, edgeColor);
  int lineStep = direction == RingDirection::Horizontal ? 1 : 8;
  int lineLen = direction == RingDirection::Horizontal ? 8 : 1;

  auto getColor = [=](int idx) {
    return selectingCurrent == -1 ? normalCharColor: idx == selectingCurrent ? selectingCharColor : unSelectingCharColor;
  };

  if (selectingOther == -1) {
    auto offsets = calcOffsets(1);
    for (int pos = 0; pos < 8; ++pos) {
      int colOrigin = lineStep * pos;
      auto ringColor = getColor(pos);
      renderRingChars(ftRenderer, offsets[pos], 0.2f, [=, &chars](int idx) -> std::pair<std::u8string&, glm::vec3> {
        return { chars[colOrigin + lineLen * idx], ringColor };
      });
    }
  } else {
    int lineOrigin = lineLen * selectingOther;
    renderRingChars(ftRenderer, {0, 0}, 1, [=, &chars, &getColor](auto idx) -> std::pair<std::u8string&, glm::vec3> {
      return {chars[lineOrigin + lineStep * idx], getColor(idx)};
    });
  }
  ftRenderer.doDraw();

  ccRenderer.draw(stickPos);
}

RingRenderer::RingRenderer(
    FreetypeRenderer &ftRenderer,
    CursorCircleRenderer &ccRenderer,
    BackgroundRingRenderer &brRenderer,
    const glm::vec3 &normalCharColor,
    const glm::vec3 &unSelectingCharColor,
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
    normalCharColor(normalCharColor),
    unSelectingCharColor(unSelectingCharColor),
    selectingCharColor(selectingCharColor) {}