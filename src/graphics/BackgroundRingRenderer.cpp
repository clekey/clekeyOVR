//
// Created by anatawa12 on 2022/09/03.
//

#include "BackgroundRingRenderer.h"
#include "glutil.h"
#include <memory>

std::unique_ptr<BackgroundRingRenderer> BackgroundRingRenderer::create() {
  gl::Program program = std::move(compile_shader_program(
      "#version 330 core\n"
      "layout(location = 0) in vec2 position;\n"
      "out vec2 xy;\n"
      "void main() {\n"
      "    gl_Position.xy = position;\n"
      "    xy = position;\n"
      "}\n",
      "#version 330 core\n"
      "in vec2 xy;\n"
      "// uniforms\n"
      "uniform vec4 uCenterColor;"
      "uniform vec4 uBackgroundColor;"
      "uniform vec4 uEdgeColor;"
      "// Ouput data\n"
      "out vec4 color;\n"
      "\n"
      "// constants\n"
      "const float SIN_PI_8 = 0.38268343236; //sin(PI / 8)\n"
      "const float COS_PI_8 = 0.92387953251; //cos(PI / 8)\n"
      "const float EDGE_WIDTH_2 = 10.0/512.0;\n"
      "\n"
      "float pow2(float x) { return x * x; }\n"
      "\n"
      "bool is_background_edge() {\n"
      "    return abs(SIN_PI_8 * xy.x - COS_PI_8 * xy.y) < EDGE_WIDTH_2\n"
      "        || abs(SIN_PI_8 * xy.x + COS_PI_8 * xy.y) < EDGE_WIDTH_2\n"
      "        || abs(COS_PI_8 * xy.x - SIN_PI_8 * xy.y) < EDGE_WIDTH_2\n"
      "        || abs(COS_PI_8 * xy.x + SIN_PI_8 * xy.y) < EDGE_WIDTH_2\n"
      "        ;\n"
      "}\n"
      "\n"
      "void main() {\n"
      "    float len_sqrt = dot(xy, xy);\n"
      "    color = \n"
      "        len_sqrt < pow2(128.0/256.0) ? uCenterColor\n"
      "      : len_sqrt < pow2(246.0/256.0) ? (is_background_edge() ? uEdgeColor : uBackgroundColor)\n"
      "      : len_sqrt < pow2(256.0/256.0) ? uEdgeColor\n"
      "      : vec4(0, 0, 0, 0)\n"
      "      ;\n"
      "}\n"
  ));
  gl::VertexAttrib vertexPositionAttrib(program, "position");

  gl::VertexArray vertexArray;
  gl::ArrayBuffer vertexBuffer;

  static const GLfloat g_vertex_buffer_data[] = {
      -1.0f, -1.0f,
      +1.0f, -1.0f,
      +1.0f, +1.0f,

      -1.0f, -1.0f,
      +1.0f, +1.0f,
      -1.0f, +1.0f,
  };
  gl::Bind(vertexBuffer);
  vertexBuffer.data(sizeof(g_vertex_buffer_data), g_vertex_buffer_data, gl::kStaticDraw);

  gl::Bind(program);
  // colors
  gl::Uniform<glm::vec4> uCenterColor(program, "uCenterColor");
  gl::Uniform<glm::vec4> uBackgroundColor(program, "uBackgroundColor");
  gl::Uniform<glm::vec4> uEdgeColor(program, "uEdgeColor");

  gl::Bind(vertexArray);
  vertexPositionAttrib.enable();
  gl::Bind(vertexBuffer);
  vertexPositionAttrib.pointer(2, gl::kFloat, false, 0, nullptr);

  auto res = new BackgroundRingRenderer{
      .program = std::move(program),
      .vertexPositionAttrib = std::move(vertexPositionAttrib),
      .vertexArray = std::move(vertexArray),
      .vertexBuffer = std::move(vertexBuffer),
      .uCenterColor = std::move(uCenterColor),
      .uBackgroundColor = std::move(uBackgroundColor),
      .uEdgeColor = std::move(uEdgeColor),
  };
  return std::unique_ptr<BackgroundRingRenderer>(res);
}

void BackgroundRingRenderer::draw(
    glm::vec4 centerColor,
    glm::vec4 backgroundColor,
    glm::vec4 edgeColor
) {
  gl::Bind(vertexArray);
  gl::Use(program);

  uCenterColor.set(centerColor);
  uBackgroundColor.set(backgroundColor);
  uEdgeColor.set(edgeColor);

  gl::DrawArrays(gl::kTriangles, 0, 6);

  check_gl_err("drawing backgroundColor gui");
}
