//
// Created by anatawa12 on 8/11/22.
//

#ifndef CLEKEY_OVR_MAINGUIRENDERER_H
#define CLEKEY_OVR_MAINGUIRENDERER_H

#include <GL/glew.h>
#include "oglwrap/oglwrap.h"

class MainGuiRenderer {
public:
  static MainGuiRenderer create(int width, int height);

  void draw();

  int width, height;

  gl::Program shader_program;
  gl::VertexAttrib vertexPositionAttrib;
  gl::VertexAttrib colorAttrib;

  gl::Texture2D dest_texture;
  gl::Renderbuffer depth_buffer;
  gl::Framebuffer frame_buffer;

  gl::VertexArray vertex_array;
  gl::ArrayBuffer vertexbuffer;
};

#endif //CLEKEY_OVR_MAINGUIRENDERER_H
