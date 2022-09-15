//
// Created by anatawa12 on 8/11/22.
//

#include "MainGuiRenderer.h"
#include "glutil.h"
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

}

std::unique_ptr<MainGuiRenderer> MainGuiRenderer::create(glm::ivec2 size) {
  gl::Renderbuffer depth_buffer;
  gl::Framebuffer frame_buffer;

  gl::Bind(frame_buffer);

  gl::Bind(depth_buffer);
  depth_buffer.storage(gl::kDepthComponent, size.x, size.y);
  frame_buffer.attachBuffer(gl::kDepthAttachment, depth_buffer);
  //frame_buffer.attachTexture(gl::kColorAttachment0, dest_texture, 0);

  gl::DrawBuffers({gl::kColorAttachment0});

  gl::FramebufferStatus buffer_status = frame_buffer.status();
  if (buffer_status != gl::kFramebufferComplete) {
    std::cerr << "GL_FRAMEBUFFER mismatch: " << GLenum(buffer_status) << std::endl;
  }
  check_gl_err("rendered_texture generation");

  gl::ClearColor(0.0f, 0.0f, 0.0f, 0.0f);

  auto ftRenderer = FreetypeRenderer::create();
  auto backgroundRingRenderer = BackgroundRingRenderer::create();
  auto cursorCircleRenderer = CursorCircleRenderer::create();

  std::cout << "loading fonts" << std::endl;
  for (const auto &entry : std::filesystem::directory_iterator("./fonts")) {
    if (entry.path().extension() == ".otf" || entry.path().extension() == ".ttf") {
      ftRenderer->addFontType(entry.path().string().c_str());
      std::cout << "loaded font:" << entry.path() << std::endl;
    }
  }

  auto res = new MainGuiRenderer{
      .size = size,
      .depth_buffer = std::move(depth_buffer),
      .frame_buffer = std::move(frame_buffer),

      .backgroundRingRenderer = std::move(backgroundRingRenderer),
      .cursorCircleRenderer = std::move(cursorCircleRenderer),
      .ftRenderer = std::move(ftRenderer),
  };
  return std::unique_ptr<MainGuiRenderer>(res);
}

void MainGuiRenderer::drawRing(
    const AppStatus &status,
    LeftRight side,
    bool alwaysShowInCircle,
    gl::Texture2D& texture
) {
  gl::Bind(frame_buffer);
  frame_buffer.attachTexture(gl::kColorAttachment0, texture, 0);
  gl::Viewport(0, 0, size.x, size.y);
  gl::Clear().Color().Depth();
  gl::Enable(gl::kBlend);
  gl::BlendFunc(gl::kSrcAlpha, gl::kOneMinusSrcAlpha);


  int8_t selectingCurrent = status.getSelectingOfCurrentSide(side);
  int8_t selectingOpposite = status.getSelectingOfOppositeSide(side);

  auto stickPos = status.getStickPos(side);
  glm::vec3 normalCharColor = {0.0, 0.0, 0.0};
  glm::vec3 unSelectingCharColor = {0.5, 0.5, 0.5};
  glm::vec3 selectingCharColor = {0.0, 0.0, 0.0};

  backgroundRingRenderer->draw();

  int lineStep = side == LeftRight::Left ? 8 : 1;
  int lineLen = side == LeftRight::Left ? 1 : 8;

  auto getColor = [=](int idx) {
    return selectingCurrent == -1
           ? normalCharColor
           : idx == selectingCurrent
             ? selectingCharColor
             : unSelectingCharColor;
  };

  if (alwaysShowInCircle || selectingOpposite == -1) {
    auto offsets = calcOffsets(1);
    for (int pos = 0; pos < 8; ++pos) {
      int colOrigin = lineStep * pos;
      auto ringColor = getColor(pos);
      renderRingChars(*ftRenderer, offsets[pos], 0.2f, [=](int idx) -> std::pair<const std::u8string &, glm::vec3> {
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
}
