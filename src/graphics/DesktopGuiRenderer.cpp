//
// Created by anatawa12 on 8/11/22.
//

#include "DesktopGuiRenderer.h"
#include "glutil.h"

DesktopGuiRenderer::DesktopGuiRenderer(int width, int height) :
        width(width),
        height(height),
        shader_program((gl::Unbind(gl::kFramebuffer), std::move(compile_shader_program(
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
        )))),
        vertexPositionAttrib(shader_program, "vertexPosition_modelspace"),
        texture_id((gl::Bind(shader_program), shader_program), "rendered_texture") {

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
}

void DesktopGuiRenderer::draw(gl::Texture2D &texture) {
    // スクリーンに描画する。
    gl::Unbind(gl::kFramebuffer);
    gl::Bind(vertex_array);

    glViewport(0, 0, width, height);
    gl::Clear().Color().Depth();
    gl::Use(shader_program);

    gl::BindToTexUnit(texture, 0);
    texture_id.set(0);

    vertexPositionAttrib.enable();
    gl::Bind(vertex_buffer);
    vertexPositionAttrib.pointer(3, gl::kFloat, false, 0, nullptr);
    gl::DrawArrays(gl::kTriangles, 0, 6);
    vertexPositionAttrib.disable();

    check_gl_err("drawing desktop gui");
}