//
// Created by anatawa12 on 2022/09/03.
//

#ifndef CLEKEY_OVR_CURSORCIRCLERENDERER_H
#define CLEKEY_OVR_CURSORCIRCLERENDERER_H


#include <GL/glew.h>
#include "oglwrap/oglwrap.h"

class CursorCircleRenderer {
public:
  static std::unique_ptr<CursorCircleRenderer> create();

  void draw(
      glm::vec2 center,
      glm::vec2 size,
      glm::vec2 stick,
      glm::vec4 color = glm::vec4(0.22, 0.22, 0.22, 1.0)
  );

  gl::Program program;
  gl::VertexAttrib vertexPositionAttrib;
  gl::VertexArray vertexArray;
  gl::ArrayBuffer vertexBuffer;

  // transform
  gl::Uniform<glm::vec2> uCenter;
  gl::Uniform<glm::vec2> uSize;
  // colors
  gl::Uniform<glm::vec4> uStickColor;
  gl::Uniform<glm::vec2> uStickPos;
};


#endif //CLEKEY_OVR_CURSORCIRCLERENDERER_H
