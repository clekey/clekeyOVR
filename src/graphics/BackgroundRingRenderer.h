//
// Created by anatawa12 on 2022/09/03.
//

#ifndef CLEKEY_OVR_BACKGROUNDRINGRENDERER_H
#define CLEKEY_OVR_BACKGROUNDRINGRENDERER_H


#include <GL/glew.h>
#include "oglwrap/oglwrap.h"

class BackgroundRingRenderer {
public:
  static BackgroundRingRenderer create();

  void draw(
      glm::vec2 center,
      glm::vec2 size,
      glm::vec4 centerColor = glm::vec4(0.83, 0.83, 0.83, 1.0),
      glm::vec4 backgroundColor = glm::vec4(0.686, 0.686, 0.686, 1.0),
      glm::vec4 edgeColor = glm::vec4(1.0, 1.0, 1.0, 1.0)
  );

  gl::Program program;
  gl::VertexAttrib vertexPositionAttrib;
  gl::VertexArray vertexArray;
  gl::ArrayBuffer vertexBuffer;

  // transform
  gl::Uniform<glm::vec2> uCenter;
  gl::Uniform<glm::vec2> uSize;
  // colors
  gl::Uniform<glm::vec4> uCenterColor;
  gl::Uniform<glm::vec4> uBackgroundColor;
  gl::Uniform<glm::vec4> uEdgeColor;
};


#endif //CLEKEY_OVR_BACKGROUNDRINGRENDERER_H
