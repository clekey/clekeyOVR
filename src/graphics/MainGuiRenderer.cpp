//
// Created by anatawa12 on 8/11/22.
//

#include "MainGuiRenderer.h"
#include "glutil.h"
#include "RingRenderer.h"

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
  std::unique_ptr<RingRenderer> ringRenderer(new RingRenderer(
      *ftRenderer,
      *cursorCircleRenderer,
      *backgroundRingRenderer,
      {0.0, 0.0, 0.0},
      {0.5, 0.5, 0.5},
      {0.0, 0.0, 0.0}
  ));

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
      .ringRenderer = std::move(ringRenderer),
  };
  return std::unique_ptr<MainGuiRenderer>(res);
}

void MainGuiRenderer::draw(const OVRController &controller, LeftRight side) {
  gl::Bind(frame_buffer);
  frame_buffer.attachTexture(gl::kColorAttachment0, dest_textures[side], 0);
  gl::Viewport(0, 0, width, height);
  gl::Clear().Color().Depth();
  gl::Enable(gl::kBlend);
  gl::BlendFunc(gl::kSrcAlpha, gl::kOneMinusSrcAlpha);


  float size = 2;
  auto ringDir = side == LeftRight::Left ? RingDirection::Horizontal : RingDirection::Vertical;
  int selectingCurrent = 1;
  int selectingOther = -1;
  if (side == LeftRight::Right) std::swap(selectingCurrent, selectingOther);

  ringRenderer->render({0, 0}, controller.getStickPos(side), size, ringDir, selectingCurrent, selectingOther, {
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
