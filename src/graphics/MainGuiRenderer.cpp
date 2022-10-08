//
// Created by anatawa12 on 8/11/22.
//

#include <include/core/SkCanvas.h>
#include "MainGuiRenderer.h"
#include "glutil.h"
#include "../global.h"
#include <array>
#include <filesystem>

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

#if 0
void renderRingChars(FreetypeRenderer &renderer, glm::vec2 center, float size,
                     std::function<std::pair<const std::u8string &, glm::vec3>(int)> getChar) {
  float fontSize = size * 0.4f;
  auto offsets = calcOffsets(size);

  for (int i = 0; i < 8; ++i) {
    auto pair = getChar(i);
    renderer.addCenteredStringWithMaxWidth(
        pair.first, center + offsets[i], pair.second,
        {fontSize, fontSize}, fontSize,
        CenteredMode::Both);
  }
}
#endif

}

std::unique_ptr<MainGuiRenderer> MainGuiRenderer::create(glm::ivec2 size) {
  //auto ftRenderer = FreetypeRenderer::create();
  auto backgroundRingRenderer = BackgroundRingRenderer::create();
  //auto cursorCircleRenderer = CursorCircleRenderer::create();

  //std::cout << "loading fonts" << std::endl;
  //for (const auto &entry: std::filesystem::directory_iterator(getResourcesDir() / "fonts")) {
  //  if (entry.path().extension() == ".otf" || entry.path().extension() == ".ttf") {
  //    ftRenderer->addFontType(entry.path().string().c_str());
  //    std::cout << "loaded font:" << entry.path() << std::endl;
  //  }
  //}

  auto res = new MainGuiRenderer{
      .size = size,

      .backgroundRingRenderer = std::move(backgroundRingRenderer),
      //.cursorCircleRenderer = std::move(cursorCircleRenderer),
      //.ftRenderer = std::move(ftRenderer),
  };
  return std::unique_ptr<MainGuiRenderer>(res);
}

void MainGuiRenderer::drawRing(
    const KeyboardStatus &status,
    LeftRight side,
    bool alwaysShowInCircle,
    const RingOverlayConfig &config,
    SkSurface& surface
) {
  // clear to transparent
  surface.getCanvas()->clear(SK_ColorTRANSPARENT);
  check_gl_err("drawRing: clear");

  int8_t selectingCurrent = status.getSelectingOfCurrentSide(side);
  int8_t selectingOpposite = status.getSelectingOfOppositeSide(side);

  auto stickPos = status.getStickPos(side);

  backgroundRingRenderer->draw(
      surface,
      {config.centerColor, 1},
      {config.backgroundColor, 1},
      {config.edgeColor, 1}
  );
  check_gl_err("drawRing: background");

#if 0
  int lineStep = side == LeftRight::Left ? 8 : 1;
  int lineLen = side == LeftRight::Left ? 1 : 8;

  auto getColor = [selectingCurrent, &config](int idx) {
    return selectingCurrent == -1
           ? config.normalCharColor
           : idx == selectingCurrent
             ? config.selectingCharColor
             : config.unSelectingCharColor;
  };

  if (alwaysShowInCircle || selectingOpposite == -1) {
    auto offsets = calcOffsets(1);
    for (int pos = 0; pos < 8; ++pos) {
      int colOrigin = lineStep * pos;
      auto ringColor = getColor(pos);
      auto ringSize = pos == selectingCurrent ? 0.22f : 0.2f;
      renderRingChars(*ftRenderer, offsets[pos], ringSize, [=](int idx) -> std::pair<const std::u8string &, glm::vec3> {
        return {status.method->getTable()[colOrigin + lineLen * idx], ringColor};
      });
    }
  } else {
    int lineOrigin = lineLen * selectingOpposite;
    renderRingChars(*ftRenderer, {0, 0}, 1,
                    [=, &status, &getColor](auto idx) -> std::pair<const std::u8string &, glm::vec3> {
                      return {status.method->getTable()[lineOrigin + lineStep * idx], getColor(idx)};
                    });
  }
  ftRenderer->doDraw();

  cursorCircleRenderer->draw(stickPos);

  gl::Unbind(frame_buffer);

  check_gl_err("main gui rendering");
#endif
}

void MainGuiRenderer::drawCenter(
    const KeyboardStatus &status,
    const CompletionOverlayConfig &config,
    sk_sp<SkSurface> texture
) {
#if 0
  gl::Bind(frame_buffer);
  frame_buffer.attachTexture(gl::kColorAttachment0, texture, 0);
  gl::Viewport(0, 0, size.x, size.y / 8);
  gl::ClearColor(config.backgroundColor.r, config.backgroundColor.g, config.backgroundColor.b, 1.0f);
  gl::Clear().Color().Depth();
  gl::Enable(gl::kBlend);
  gl::BlendFunc(gl::kSrcAlpha, gl::kOneMinusSrcAlpha);

  glm::vec2 fontSize{1.0f / 8, 1};

  glm::vec2 cursor{-1 + 1.0f / 8 / 2, -0.4f};

  cursor.x = ftRenderer->addString(status.method->getBuffer(), cursor, config.inputtingCharColor, fontSize);

  ftRenderer->doDraw();

  gl::Unbind(frame_buffer);

  check_gl_err("main gui rendering");
#endif
}
