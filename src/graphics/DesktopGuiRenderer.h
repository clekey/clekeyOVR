//
// Created by anatawa12 on 8/11/22.
//

#ifndef CLEKEY_OVR_DESKTOPGUIRENDERER_H
#define CLEKEY_OVR_DESKTOPGUIRENDERER_H

//#include <GL/glew.h>
#include <glm/glm.hpp>
#include <memory>

class DesktopGuiRenderer {
public:
  static std::unique_ptr<DesktopGuiRenderer> create(glm::ivec2 size);

  void preDraw();

  //void drawTexture(const gl::Texture2D &texture, glm::vec2 bottomLeft, glm::vec2 size);

  glm::ivec2 size;

  //gl::Program shader_program;
  //gl::VertexAttrib posAttrib;
  //gl::Uniform<glm::vec2> uBottomLeft;
  //gl::Uniform<glm::vec2> uSize;
  //gl::UniformSampler texture_id;
  //gl::VertexArray vertex_array;
  //gl::ArrayBuffer vertex_buffer;
};


#endif //CLEKEY_OVR_DESKTOPGUIRENDERER_H
