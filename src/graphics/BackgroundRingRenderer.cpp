//
// Created by anatawa12 on 2022/09/03.
//

#include "BackgroundRingRenderer.h"
#include "glutil.h"

BackgroundRingRenderer BackgroundRingRenderer::create() {
  gl::Program program = std::move(compile_shader_program(
      "#version 330 core\n"
      "layout(location = 0) in vec2 position;\n"
      "uniform vec2 origin;"
      "uniform vec2 size;"
      "out vec2 xy;\n"
      "void main() {\n"
      "    gl_Position.xy = position * size + origin;\n"
      "    xy = position * 2 - vec2(1, 1);\n"
      "}\n",
      "#version 330 core\n"
      "in vec2 xy;\n"
      "// uniforms\n"
      "uniform vec4 center;"
      "uniform vec4 background;"
      "uniform vec4 edge;"
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
      "        len_sqrt < pow2(128.0/256.0) ? center\n"
      "      : len_sqrt < pow2(246.0/256.0) ? (is_background_edge() ? edge : background)\n"
      "      : len_sqrt < pow2(256.0/256.0) ? edge\n"
      "      : vec4(0, 0, 0, 0)\n"
      "      ;\n"
      "}\n"
  ));
  gl::VertexAttrib vertexPositionAttrib(program, "position");

  gl::VertexArray vertexArray;
  gl::ArrayBuffer vertexBuffer;

  static const GLfloat g_vertex_buffer_data[] = {
      0.0f, 0.0f,
      1.0f, 0.0f,
      1.0f, 1.0f,

      0.0f, 0.0f,
      1.0f, 1.0f,
      0.0f, 1.0f,
  };
  gl::Bind(vertexBuffer);
  vertexBuffer.data(sizeof(g_vertex_buffer_data), g_vertex_buffer_data, gl::kStaticDraw);

  gl::Bind(program);
  // transform
  gl::Uniform<glm::vec2> origin(program, "origin");
  gl::Uniform<glm::vec2> size(program, "size");
  // colors
  gl::Uniform<glm::vec4> center(program, "center");
  gl::Uniform<glm::vec4> background(program, "background");
  gl::Uniform<glm::vec4> edge(program, "edge");

  gl::Bind(vertexArray);
  vertexPositionAttrib.enable();
  gl::Bind(vertexBuffer);
  vertexPositionAttrib.pointer(2, gl::kFloat, false, 0, nullptr);

  return BackgroundRingRenderer{
      .program = std::move(program),
      .vertexPositionAttrib = std::move(vertexPositionAttrib),
      .vertexArray = std::move(vertexArray),
      .vertexBuffer = std::move(vertexBuffer),
      .uOrigin = std::move(origin),
      .uSize = std::move(size),
      .uCenter = std::move(center),
      .uBackground = std::move(background),
      .uEdge = std::move(edge),
  };
}

void BackgroundRingRenderer::draw(
    glm::vec2 origin,
    glm::vec2 size,
    glm::vec4 center,
    glm::vec4 background,
    glm::vec4 edge
) {
  gl::Bind(vertexArray);
  gl::Use(program);

  uOrigin.set(origin);
  uSize.set(size);
  uCenter.set(center);
  uBackground.set(background);
  uEdge.set(edge);

  gl::DrawArrays(gl::kTriangles, 0, 6);

  check_gl_err("drawing background gui");
}
