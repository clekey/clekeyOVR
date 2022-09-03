//
// Created by anatawa12 on 8/11/22.
//

#include "MainGuiRenderer.h"
#include "glutil.h"

MainGuiRenderer MainGuiRenderer::create(int width, int height) {
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

  return {
      .width = width,
      .height = height,
      .dest_texture = std::move(dest_texture),
      .depth_buffer = std::move(depth_buffer),
      .frame_buffer = std::move(frame_buffer),

      .backgroundRingRenderer = BackgroundRingRenderer::create(),
  };
}

void MainGuiRenderer::draw() {
  gl::Bind(frame_buffer);
  gl::Viewport(0, 0, width, height);
  gl::Clear().Color().Depth();
  gl::Enable(gl::kBlend);
  gl::BlendFunc(gl::kSrcAlpha, gl::kOneMinusSrcAlpha);

  backgroundRingRenderer.draw(0, -1, 1, 2);
  backgroundRingRenderer.draw(-.5, -1, 1, 2);

  gl::Unbind(frame_buffer);

  check_gl_err("main gui rendering");
}
