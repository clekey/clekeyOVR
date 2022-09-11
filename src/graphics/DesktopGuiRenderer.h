//
// Created by anatawa12 on 8/11/22.
//

#ifndef CLEKEY_OVR_DESKTOPGUIRENDERER_H
#define CLEKEY_OVR_DESKTOPGUIRENDERER_H

#include <GL/glew.h>
#include <oglwrap/oglwrap.h>

class DesktopGuiRenderer {
public:
  static std::unique_ptr<DesktopGuiRenderer> create(int width, int height);

  void draw(const gl::Texture2D &texture);

  int width, height;

  gl::Program shader_program;
  gl::VertexAttrib vertexPositionAttrib;
  gl::UniformSampler texture_id;
  gl::VertexArray vertex_array;
  gl::ArrayBuffer vertex_buffer;
};


#endif //CLEKEY_OVR_DESKTOPGUIRENDERER_H
