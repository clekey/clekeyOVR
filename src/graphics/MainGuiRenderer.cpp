//
// Created by anatawa12 on 8/11/22.
//

#include "MainGuiRenderer.h"
#include "glutil.h"

MainGuiRenderer::MainGuiRenderer(int width, int height) :
        width(width),
        height(height),
        shader_program(std::move(compile_shader_program(
                "#version 330 core\n"
                "layout(location = 0) in vec3 vertexPosition_modelspace;\n"
                "layout(location = 1) in vec4 color;\n"
                "out vec4 out_color;\n"
                "void main() {\n"
                "    gl_Position.xyz = vertexPosition_modelspace;\n"
                "    out_color = color;\n"
                "}\n",
                "#version 330 core\n"
                "in vec4 out_color;\n"
                ""
                "// Ouput data\n"
                "layout(location = 0) out vec4 color;\n"
                "\n"
                "void main() {\n"
                "    // Output color = red \n"
                "    color = out_color;\n"
                "}\n"
        ))),
        vertexPositionAttrib(shader_program, "vertexPosition_modelspace"),
        colorAttrib(shader_program, "color")
{
     gl::Bind(frame_buffer);

    gl::Bind(dest_texture);
    dest_texture.upload(
            gl::kRgba8, width, height,
            gl::kRgb, gl::kUnsignedByte, nullptr
    );
    dest_texture.magFilter(gl::kNearest);
    dest_texture.minFilter(gl::kNearest);

    gl::Bind(depth_buffer);
    depth_buffer.storage(gl::kDepthComponent, width, height);
    frame_buffer.attachBuffer(gl::kDepthAttachment, depth_buffer);

    frame_buffer.attachTexture(gl::kColorAttachment0, dest_texture, 0);

    gl::DrawBuffers({gl::kColorAttachment0});

    gl::FramebufferStatus buffer_status = frame_buffer.status();
    if (buffer_status != gl::kFramebufferComplete) {
        std::cerr << "GL_FRAMEBUFFER mismatch: " << GLenum(buffer_status) << std::endl;
    }
    check_gl_err("rendered_texture generation");

    gl::ClearColor(0.0f, 0.0f, 0.0f, 0.0f);

    static const GLfloat g_vertex_buffer_data[] = {
            -1.0f, -1.0f, 0.0f,
            1.0f, -1.0f, 0.0f,
            0.0f, 1.0f, 0.0f,
    };
    gl::Bind(vertexbuffer);
    vertexbuffer.data(sizeof(g_vertex_buffer_data), g_vertex_buffer_data, gl::kStaticDraw);

    static const GLfloat g_color_buffer_data[] = {
            1.0f, 0.0f, 0.0f, 1.0f,
            0.0f, 1.0f, 0.0f, 1.0f,
            0.0f, 0.0f, 1.0f, 1.0f,
    };
    gl::Bind(colorbuffer);
    colorbuffer.data(sizeof(g_color_buffer_data), g_color_buffer_data, gl::kStaticDraw);
}

void MainGuiRenderer::draw() {
    gl::Bind(rendered_textures[0].frame_buffer);
    gl::Bind(vertex_array);
    gl::Viewport(0, 0, width, height);
    gl::Clear().Color().Depth();

    gl::Use(shader_program);

    // 1rst attribute buffer : vertices
    vertexPositionAttrib.enable();
    gl::Bind(vertexbuffer);
    vertexPositionAttrib.pointer(3, gl::kFloat, false, 0, nullptr);
    colorAttrib.enable();
    gl::Bind(colorbuffer);
    colorAttrib.pointer(4, gl::kFloat, false, 0, nullptr);
    // Draw the triangle !
    gl::DrawArrays(gl::kTriangles, 0, 3);
    vertexPositionAttrib.disable();
    colorAttrib.disable();

    gl::Unbind(rendered_textures[0].frame_buffer);

    check_gl_err("main gui rendering");
}
