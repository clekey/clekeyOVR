//
// Created by anatawa12 on 8/11/22.
//

#include "MainGuiRenderer.h"
#include "glutil.h"
#include <array>

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

std::unique_ptr<MainGuiRenderer> MainGuiRenderer::create(int width, int height) {
  gl::Texture2D dest_textures[2];
  gl::Renderbuffer depth_buffer;
  gl::Framebuffer frame_buffer;

  gl::Bind(frame_buffer);

  // TODO: consider to have those texture on glmain instead of renderer?
  for (auto &dest_texture: dest_textures) {
    gl::Bind(dest_texture);
    dest_texture.upload(
        gl::kRgba8, width, height,
        gl::kRgb, gl::kUnsignedByte, nullptr
    );
    dest_texture.magFilter(gl::kLinear);
    dest_texture.minFilter(gl::kLinear);
  }

  gl::Bind(depth_buffer);
  depth_buffer.storage(gl::kDepthComponent, width, height);
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

  ftRenderer->addFontType("./fonts/NotoSansJP-Medium.otf");

  auto res = new MainGuiRenderer{
      .width = width,
      .height = height,
      .dest_textures = {std::move(dest_textures[0]), std::move(dest_textures[1])},
      .depth_buffer = std::move(depth_buffer),
      .frame_buffer = std::move(frame_buffer),

      .backgroundRingRenderer = std::move(backgroundRingRenderer),
      .cursorCircleRenderer = std::move(cursorCircleRenderer),
      .ftRenderer = std::move(ftRenderer),
  };
  return std::unique_ptr<MainGuiRenderer>(res);
}

// currently internal in this file but will be moved to header
enum class RingDirection {
  Horizontal,
  Vertical,
};

void MainGuiRenderer::draw(const OVRController &controller, LeftRight side) {
  gl::Bind(frame_buffer);
  frame_buffer.attachTexture(gl::kColorAttachment0, dest_textures[side], 0);
  gl::Viewport(0, 0, width, height);
  gl::Clear().Color().Depth();
  gl::Enable(gl::kBlend);
  gl::BlendFunc(gl::kSrcAlpha, gl::kOneMinusSrcAlpha);


  auto direction = side == LeftRight::Left ? RingDirection::Horizontal : RingDirection::Vertical;
  int selectingCurrent = 1;
  int selectingOther = -1;
  if (side == LeftRight::Right) std::swap(selectingCurrent, selectingOther);

  std::u8string chars[] = {
      u8"A", u8"A", u8"A", u8"A", u8"A", u8"A", u8"A", u8"A",
      u8"a", u8"a", u8"a", u8"a", u8"a", u8"a", u8"a", u8"a",
      u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042",
      u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044",
      u8"C", u8"C", u8"C", u8"C", u8"C", u8"C", u8"C", u8"C",
      u8"c", u8"c", u8"c", u8"c", u8"c", u8"c", u8"c", u8"c",
      u8"D", u8"D", u8"D", u8"D", u8"D", u8"D", u8"D", u8"D",
      u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=",
  };
  auto stickPos = controller.getStickPos(side);
  glm::vec3 normalCharColor = {0.0, 0.0, 0.0};
  glm::vec3 unSelectingCharColor = {0.5, 0.5, 0.5};
  glm::vec3 selectingCharColor = {0.0, 0.0, 0.0};

  backgroundRingRenderer->draw();

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
      renderRingChars(*ftRenderer, offsets[pos], 0.2f, [=, &chars](int idx) -> std::pair<std::u8string&, glm::vec3> {
        return { chars[colOrigin + lineLen * idx], ringColor };
      });
    }
  } else {
    int lineOrigin = lineLen * selectingOther;
    renderRingChars(*ftRenderer, {0, 0}, 1, [=, &chars, &getColor](auto idx) -> std::pair<std::u8string&, glm::vec3> {
      return {chars[lineOrigin + lineStep * idx], getColor(idx)};
    });
  }
  ftRenderer->doDraw();

  cursorCircleRenderer->draw(stickPos);

  gl::Unbind(frame_buffer);

  check_gl_err("main gui rendering");
}
