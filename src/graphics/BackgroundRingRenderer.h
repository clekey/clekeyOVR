//
// Created by anatawa12 on 2022/09/03.
//

#ifndef CLEKEY_OVR_BACKGROUNDRINGRENDERER_H
#define CLEKEY_OVR_BACKGROUNDRINGRENDERER_H


#include <GL/glew.h>
#include "oglwrap/oglwrap.h"

class BackgroundRingRenderer {
public:
  static std::unique_ptr<BackgroundRingRenderer> create();

  void draw(glm::vec4 centerColor, glm::vec4 backgroundColor, glm::vec4 edgeColor);

  gl::Program program;
  gl::VertexAttrib vertexPositionAttrib;
  gl::VertexArray vertexArray;
  gl::ArrayBuffer vertexBuffer;

  // colors
  gl::Uniform<glm::vec4> uCenterColor;
  gl::Uniform<glm::vec4> uBackgroundColor;
  gl::Uniform<glm::vec4> uEdgeColor;
};


#endif //CLEKEY_OVR_BACKGROUNDRINGRENDERER_H
