//
// Created by anatawa12 on 2022/09/03.
//

#ifndef CLEKEY_OVR_BACKGROUNDRINGRENDERER_H
#define CLEKEY_OVR_BACKGROUNDRINGRENDERER_H


//#include <GL/glew.h>
#include <include/core/SkSurface.h>
#include <glm/glm.hpp>

class BackgroundRingRenderer {
public:
  static std::unique_ptr<BackgroundRingRenderer> create();

  void draw(SkSurface& surface, glm::vec4 centerColor, glm::vec4 backgroundColor, glm::vec4 edgeColor);
};


#endif //CLEKEY_OVR_BACKGROUNDRINGRENDERER_H
