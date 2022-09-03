//
// Created by anatawa12 on 2022/09/03.
//

#ifndef CLEKEY_OVR_CURSORCIRCLERENDERER_H
#define CLEKEY_OVR_CURSORCIRCLERENDERER_H


#include <GL/glew.h>
#include "oglwrap/oglwrap.h"

class CursorCircleRenderer {
public:
  static CursorCircleRenderer create();

  void draw(
      float x1, float y1,
      float x2, float y2,
      float stickX, float stickY,
      glm::vec4 color = glm::vec4(0.22, 0.22, 0.22, 1.0)
  );

  gl::Program program;
  gl::VertexAttrib vertexPositionAttrib;
  gl::VertexArray vertexArray;
  gl::ArrayBuffer vertexBuffer;

  // transform
  gl::Uniform<glm::mat3> uMat;
  // colors
  gl::Uniform<glm::vec4> uStickColor;
  gl::Uniform<glm::vec2> uStickPos;
};


#endif //CLEKEY_OVR_CURSORCIRCLERENDERER_H
