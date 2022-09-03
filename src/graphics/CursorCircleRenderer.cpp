//
// Created by anatawa12 on 2022/09/03.
//

#include "CursorCircleRenderer.h"
#include "glutil.h"

CursorCircleRenderer CursorCircleRenderer::create() {
  gl::Program program = std::move(compile_shader_program(
      "#version 330 core\n"
      "layout(location = 0) in vec2 position;\n"
      "uniform mat3 mat;"
      "out vec2 xy;\n"
      "void main() {\n"
      "    gl_Position.xy = (vec3(position, 1) * mat).xy;\n"
      "    xy = position * 2 - vec2(1, 1);\n"
      "}\n",
      "#version 330 core\n"
      "in vec2 xy;\n"
      "// uniforms\n"
      "uniform vec4 stick_color;\n"
      "uniform vec2 stick_pos;\n"
      "// Ouput data\n"
      "out vec4 color;\n"
      "\n"
      "void main() {\n"
      "    vec2 diff = xy - stick_pos / 3;\n"
      "    float len_sqrt = dot(diff, diff);\n"
      "    color = len_sqrt < (0.25 * 0.25) ? stick_color : vec4(0, 0, 0, 0);\n"
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
  gl::Uniform<glm::mat3> mat(program, "mat");
  // colors
  gl::Uniform<glm::vec4> stick_color(program, "stick_color");
  gl::Uniform<glm::vec2> stick_pos(program, "stick_pos");

  return CursorCircleRenderer{
      .program = std::move(program),
      .vertexPositionAttrib = std::move(vertexPositionAttrib),
      .vertexArray = std::move(vertexArray),
      .vertexBuffer = std::move(vertexBuffer),
      .uMat = std::move(mat),
      .uStickColor = std::move(stick_color),
      .uStickPos = std::move(stick_pos),
  };
}

void CursorCircleRenderer::draw(
    float x1, float y1,
    float width, float height,
    float stickX, float stickY, glm::vec4 color) {
  gl::Bind(vertexArray);
  gl::Use(program);

  uMat.set(glm::mat3(
      width, 0, x1,
      0, height, y1,
      0, 0, 1
  ));
  uStickPos.set(glm::vec2(stickX, stickY));
  uStickColor.set(color);

  vertexPositionAttrib.enable();
  gl::Bind(vertexBuffer);
  vertexPositionAttrib.pointer(2, gl::kFloat, false, 0, nullptr);
  gl::DrawArrays(gl::kTriangles, 0, 6);
  vertexPositionAttrib.disable();

  check_gl_err("drawing background gui");
}
