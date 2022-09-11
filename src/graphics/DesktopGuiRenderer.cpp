//
// Created by anatawa12 on 8/11/22.
//

#include "DesktopGuiRenderer.h"
#include "glutil.h"

std::unique_ptr<DesktopGuiRenderer> DesktopGuiRenderer::create(int width, int height) {
  gl::Unbind(gl::kFramebuffer);
  gl::Program shader_program = std::move(compile_shader_program(
      "#version 330 core\n"
      "layout(location = 0) in vec3 vertexPosition_modelspace;\n"
      "out vec2 UV;\n"
      "void main() {\n"
      "    gl_Position.xyz = vertexPosition_modelspace;\n"
      "    UV = (vertexPosition_modelspace.xy+vec2(1,1))/2.0;\n"
      "}\n",
      "#version 330 core\n"
      "in vec2 UV;\n"
      "out vec3 color;\n"
      "\n"
      "uniform sampler2D rendered_texture;\n"
      "\n"
      "void main() {\n"
      "    color = texture(rendered_texture, UV).xyz;\n"
      //"    color = vec3(UV, 0);\n"
      "}\n"
  ));
  gl::VertexAttrib vertexPositionAttrib(shader_program, "vertexPosition_modelspace");

  gl::Bind(shader_program);
  gl::UniformSampler texture_id(shader_program, "rendered_texture");

  gl::VertexArray vertex_array;
  gl::ArrayBuffer vertex_buffer;
  static const GLfloat g_quad_vertex_buffer_data[] = {
      1.0f, -1.0f, 0.0f,
      -1.0f, -1.0f, 0.0f,
      -1.0f, 1.0f, 0.0f,

      -1.0f, 1.0f, 0.0f,
      1.0f, -1.0f, 0.0f,
      1.0f, 1.0f, 0.0f,
  };

  gl::Bind(vertex_array);
  gl::Bind(vertex_buffer);
  vertex_buffer.data(sizeof(g_quad_vertex_buffer_data), g_quad_vertex_buffer_data, gl::kStaticDraw);

  vertexPositionAttrib.enable();
  gl::Bind(vertex_buffer);
  vertexPositionAttrib.pointer(3, gl::kFloat, false, 0, nullptr);

  auto res = new DesktopGuiRenderer {
      .width = width,
      .height = height,
      .shader_program = std::move(shader_program),
      .vertexPositionAttrib = std::move(vertexPositionAttrib),
      .texture_id = std::move(texture_id),
      .vertex_array = std::move(vertex_array),
      .vertex_buffer = std::move(vertex_buffer),
  };
  return std::unique_ptr<DesktopGuiRenderer>(res);
}

void DesktopGuiRenderer::draw(const gl::Texture2D &texture) {
  // スクリーンに描画する。
  gl::Unbind(gl::kFramebuffer);
  gl::Disable(gl::kBlend);

  glViewport(0, 0, width, height);
  gl::Clear().Color().Depth();
  gl::Use(shader_program);
  gl::Bind(vertex_array);

  gl::BindToTexUnit(texture, 0);
  texture_id.set(0);

  gl::DrawArrays(gl::kTriangles, 0, 6);

  check_gl_err("drawing desktop gui");
}
