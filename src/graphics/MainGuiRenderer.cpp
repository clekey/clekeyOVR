//
// Created by anatawa12 on 8/11/22.
//

#include "MainGuiRenderer.h"
#include "glutil.h"
#include "RingRenderer.h"

std::unique_ptr<MainGuiRenderer> MainGuiRenderer::create(int width, int height) {
  gl::Texture2D dest_texture;
  gl::Renderbuffer depth_buffer;
  gl::Framebuffer frame_buffer;

  gl::Bind(frame_buffer);

  gl::Bind(dest_texture);
  dest_texture.upload(
      gl::kRgba8, width, height,
      gl::kRgb, gl::kUnsignedByte, nullptr
  );
  dest_texture.magFilter(gl::kNearest);
  dest_texture.minFilter(gl::kNearest);

  gl::Bind(depth_buffer);
  depth_buffer.storage(gl::kDepthComponent, width, height);
  frame_buffer.attachBuffer(gl::kDepthAttachment, depth_buffer);

  frame_buffer.attachTexture(gl::kColorAttachment0, dest_texture, 0);

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
  std::unique_ptr<RingRenderer> ringRenderer (new RingRenderer(
      *ftRenderer,
      *cursorCircleRenderer,
      *backgroundRingRenderer,
      {1, 0, 0},
      {1, 0, 0}
  ));

  ftRenderer->addFontType("./fonts/NotoSansJP-Medium.otf");

  auto res = new MainGuiRenderer{
      .width = width,
      .height = height,
      .dest_texture = std::move(dest_texture),
      .depth_buffer = std::move(depth_buffer),
      .frame_buffer = std::move(frame_buffer),

      .backgroundRingRenderer = std::move(backgroundRingRenderer),
      .cursorCircleRenderer = std::move(cursorCircleRenderer),
      .ftRenderer = std::move(ftRenderer),
      .ringRenderer = std::move(ringRenderer),
  };
  return std::unique_ptr<MainGuiRenderer>(res);
}

void MainGuiRenderer::draw(const OVRController &controller) {
  gl::Bind(frame_buffer);
  gl::Viewport(0, 0, width, height);
  gl::Clear().Color().Depth();
  gl::Enable(gl::kBlend);
  gl::BlendFunc(gl::kSrcAlpha, gl::kOneMinusSrcAlpha);


  glm::vec2 left {-0.65, -.45};
  glm::vec2 right {0.65, -.45};
  float size = .5;

  ringRenderer->render(left, controller.getStickPos(LeftRight::Left), size, RingDirection::Horizontal, -1, -1, {
      u8"A", u8"A", u8"A", u8"A", u8"A", u8"A", u8"A", u8"A",
      u8"a", u8"a", u8"a", u8"a", u8"a", u8"a", u8"a", u8"a",
      u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042",
      u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044",
      u8"C", u8"C", u8"C", u8"C", u8"C", u8"C", u8"C", u8"C",
      u8"c", u8"c", u8"c", u8"c", u8"c", u8"c", u8"c", u8"c",
      u8"D", u8"D", u8"D", u8"D", u8"D", u8"D", u8"D", u8"D",
      u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=",
  });

  ringRenderer->render(right, controller.getStickPos(LeftRight::Right), size, RingDirection::Vertical, -1, 1, {
      u8"A", u8"A", u8"A", u8"A", u8"A", u8"A", u8"A", u8"A",
      u8"a", u8"a", u8"a", u8"a", u8"a", u8"a", u8"a", u8"a",
      u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042", u8"\u3042",
      u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044", u8"\u3044",
      u8"C", u8"C", u8"C", u8"C", u8"C", u8"C", u8"C", u8"C",
      u8"c", u8"c", u8"c", u8"c", u8"c", u8"c", u8"c", u8"c",
      u8"D", u8"D", u8"D", u8"D", u8"D", u8"D", u8"D", u8"D",
      u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=", u8"#+=",
  });

  gl::Unbind(frame_buffer);

  check_gl_err("main gui rendering");
}
