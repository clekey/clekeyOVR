#include <iostream>
#include "openvr.h"
#include <SDL.h>
#include <GL/glew.h>
#include <vector>
#include <sstream>
#include <filesystem>
#include <fstream>

#ifdef WIN32
#include <windows.h>
#define sleep(n) Sleep(n * 1000)
#else

#include <csignal>

#define Sleep(n) usleep(n * 1000)
#endif

#define WINDOW_CAPTION "clekeyOVR"
#define WINDOW_HEIGHT 256
#define WINDOW_WIDTH 512

// shader
GLuint compile_shader_program(const char *vertex_shader_src, const char *fragment_shader_src);

// error handling
#define check_gl_err(func) check_gl_err_impl(__LINE__, func)

void check_gl_err_impl(int line, const char * func);

void handle_input_err(vr::EVRInputError error);

void get_texture_data(GLuint texture, GLint level = 0);

vr::VRActionHandle_t action_left_stick;
vr::VRActionHandle_t action_left_click;
vr::VRActionHandle_t action_left_haptic;
vr::VRActionHandle_t action_right_stick;
vr::VRActionHandle_t action_right_click;
vr::VRActionHandle_t action_right_haptic;
vr::VRActionSetHandle_t action_set_input;

void GLAPIENTRY openglMessageCallback(
        GLenum source,
        GLenum type,
        GLuint id,
        GLenum severity,
        GLsizei length,
        const GLchar* message,
        const void* userParam )
{
    fprintf( stderr, "GL CALLBACK: %s type = 0x%x, severity = 0x%x, message = %s\n",
             ( type == GL_DEBUG_TYPE_ERROR ? "** GL ERROR **" : "" ),
             type, severity, message );
}

int main(int argc, char **argv) {
    if (SDL_Init(SDL_INIT_VIDEO)) {
        std::cerr << "sdl error: " << SDL_GetError() << std::endl;
        return -1;
    }

    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
    SDL_Window *window = SDL_CreateWindow(
            WINDOW_CAPTION,
            0, 0,
            WINDOW_WIDTH, WINDOW_HEIGHT,
            SDL_WINDOW_OPENGL);
    if (!window) {
        std::cerr << "sdl error: " << SDL_GetError() << std::endl;
        return -1;
    }

    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 1);
    SDL_GLContext context = SDL_GL_CreateContext(window);
    if (!context) return -1;

    glewExperimental = true;
    glewInit();

    glEnable(GL_DEBUG_OUTPUT);
    glDebugMessageCallback(openglMessageCallback, nullptr);

    GLuint vertex_array;
    glGenVertexArrays(1, &vertex_array);
    glBindVertexArray(vertex_array);

    GLuint shader_program = compile_shader_program(
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
    );

    GLuint texture_quad_shader_program = compile_shader_program(
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
    );
    GLint texture_id = glGetUniformLocation(texture_quad_shader_program, "rendered_texture");

    static const GLfloat g_quad_vertex_buffer_data[] = {
            1.0f, -1.0f, 0.0f,
            -1.0f, -1.0f, 0.0f,
            -1.0f,  1.0f, 0.0f,

            -1.0f,  1.0f, 0.0f,
            1.0f, -1.0f, 0.0f,
            1.0f,  1.0f, 0.0f,
    };

    GLuint quad_vertex_array;
    glGenVertexArrays(1, &quad_vertex_array);
    glBindVertexArray(quad_vertex_array);

    GLuint quad_vertex_buffer;
    glGenBuffers(1, &quad_vertex_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, quad_vertex_buffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof(g_quad_vertex_buffer_data), g_quad_vertex_buffer_data, GL_STATIC_DRAW);


    struct {
        GLuint texture;
        GLuint frame_buffer;
    } rendered_textures[1];


    for (auto & rendered_texture : rendered_textures) {
        glGenFramebuffers(1, &rendered_texture.frame_buffer);
        glBindFramebuffer(GL_FRAMEBUFFER, rendered_texture.frame_buffer);

        glGenTextures(1, &rendered_texture.texture);
        glBindTexture(GL_TEXTURE_2D, rendered_texture.texture);
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB, WINDOW_WIDTH, WINDOW_HEIGHT, 0, GL_RGB, GL_UNSIGNED_BYTE, nullptr);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);

        GLuint depth_buffer;
        glGenRenderbuffers(1, &depth_buffer);
        glBindRenderbuffer(GL_RENDERBUFFER, depth_buffer);
        glRenderbufferStorage(GL_RENDERBUFFER, GL_DEPTH_COMPONENT, WINDOW_WIDTH, WINDOW_HEIGHT);
        glFramebufferRenderbuffer(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_RENDERBUFFER, depth_buffer);

        glFramebufferTexture(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, rendered_texture.texture, 0);

        GLenum DrawBuffers[1] = {GL_COLOR_ATTACHMENT0};
        glDrawBuffers(1, DrawBuffers);

        GLenum buffer_status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
        if (buffer_status != GL_FRAMEBUFFER_COMPLETE) {
            std::cerr << "GL_FRAMEBUFFER mismatch: " << buffer_status << std::endl;
            return -1;
        }
        check_gl_err("rendered_texture generation");
    }

    // setup viewport
    glClearColor(0.0f, 0.0f, 0.0f, 0.0f);

    static const GLfloat g_vertex_buffer_data[] = {
            -1.0f, -1.0f, 0.0f,
            1.0f, -1.0f, 0.0f,
            0.0f, 1.0f, 0.0f,
    };
    GLuint vertexbuffer;
    glGenBuffers(1, &vertexbuffer);
    glBindBuffer(GL_ARRAY_BUFFER, vertexbuffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof(g_vertex_buffer_data), g_vertex_buffer_data, GL_STATIC_DRAW);

    static const GLfloat g_color_buffer_data[] = {
            1.0f, 0.0f, 0.0f, 1.0f,
            0.0f, 1.0f, 0.0f, 1.0f,
            0.0f, 0.0f, 1.0f, 1.0f,
    };
    GLuint colorbuffer;
    glGenBuffers(1, &colorbuffer);
    glBindBuffer(GL_ARRAY_BUFFER, colorbuffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof(g_color_buffer_data), g_color_buffer_data, GL_STATIC_DRAW);

    static const Uint32 interval = 1000 / 90;
    static Uint32 nextTime = SDL_GetTicks() + interval;
    std::cout << "Hello, World!" << std::endl;

    vr::HmdError err;
    vr::VR_Init(&err, vr::EVRApplicationType::VRApplication_Overlay);
    if (!vr::VROverlay()) {
        std::cerr << "error: " << vr::VR_GetVRInitErrorAsEnglishDescription(err) << std::endl;
        return -1;
    }
    handle_input_err(vr::VRInput()->SetActionManifestPath(
            R"(C:\Users\anata\clekey-ovr-build\actions.json)"));

#define GetActionHandle(name) handle_input_err(vr::VRInput()->GetActionHandle("/actions/input/in/" #name, &action_##name))
    GetActionHandle(left_stick);
    GetActionHandle(left_click);
    GetActionHandle(left_haptic);
    GetActionHandle(right_stick);
    GetActionHandle(right_click);
    GetActionHandle(right_haptic);
    handle_input_err(vr::VRInput()->GetActionSetHandle("/actions/input", &action_set_input));
#undef GetActionHandle

    std::cout << "action_left_stick:   " << action_left_stick << std::endl;
    std::cout << "action_left_click:   " << action_left_click << std::endl;
    std::cout << "action_left_haptic:  " << action_left_haptic << std::endl;
    std::cout << "action_right_stick:  " << action_right_stick << std::endl;
    std::cout << "action_right_click:  " << action_right_click << std::endl;
    std::cout << "action_right_haptic: " << action_right_haptic << std::endl;
    std::cout << "action_set_input:    " << action_set_input << std::endl;

    std::cout << "successfully launched" << std::endl;

    for (;;) {
        //*
        vr::VRActiveActionSet_t action = {};
        action.ulActionSet = action_set_input;
        handle_input_err(vr::VRInput()->UpdateActionState(&action, sizeof(vr::VRActiveActionSet_t), 1));
        vr::InputAnalogActionData_t analog_data = {};
        handle_input_err(vr::VRInput()->GetAnalogActionData(action_left_stick, &analog_data, sizeof(analog_data),
                                                            vr::k_ulInvalidInputValueHandle));
        std::cout << "left input:  " << analog_data.bActive << ": "
                  << analog_data.x << ", " << analog_data.y << std::endl;
        handle_input_err(vr::VRInput()->GetAnalogActionData(
                action_right_stick, &analog_data, sizeof(analog_data),
                vr::k_ulInvalidInputValueHandle));
        std::cout << "right input: " << analog_data.bActive << ": "
                  << analog_data.x << ", " << analog_data.y << std::endl;
        vr::InputDigitalActionData_t digital_data = {};
        handle_input_err(vr::VRInput()->GetDigitalActionData(
                action_left_click, &digital_data, sizeof(digital_data),
                vr::k_ulInvalidInputValueHandle));
        std::cout << "left click:  " << digital_data.bActive << ": "
                  << digital_data.bState << std::endl;
        handle_input_err(vr::VRInput()->GetDigitalActionData(
                action_right_click, &digital_data, sizeof(digital_data),
                vr::k_ulInvalidInputValueHandle));
        std::cout << "right click: " << digital_data.bActive << ": "
                  << digital_data.bState << std::endl;

        SDL_Event ev;
        SDL_Keycode key;
        while (SDL_PollEvent(&ev)) {
            switch (ev.type) {
                case SDL_QUIT:
                    goto quit;
                case SDL_KEYDOWN:
                    key = ev.key.keysym.sym;
                    if (key == SDLK_ESCAPE)
                        goto quit;
                    break;
            }
        }

        glBindFramebuffer(GL_FRAMEBUFFER, rendered_textures[0].frame_buffer);
        //glBindFramebuffer(GL_FRAMEBUFFER, 0);
        glViewport(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
        glClear( GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        glUseProgram(shader_program);

        // 1rst attribute buffer : vertices
        glEnableVertexAttribArray(0);
        glBindBuffer(GL_ARRAY_BUFFER, vertexbuffer);
        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 0, nullptr);
        glEnableVertexAttribArray(1);
        glBindBuffer(GL_ARRAY_BUFFER, colorbuffer);
        glVertexAttribPointer(1, 4, GL_FLOAT, GL_FALSE, 0, nullptr);
        // Draw the triangle !
        glDrawArrays(GL_TRIANGLES, 0, 3);
        glDisableVertexAttribArray(0);
        glDisableVertexAttribArray(1);

        check_gl_err("framebuffer render");

        get_texture_data(rendered_textures[0].texture, 0);

        // スクリーンに描画する。
        glBindFramebuffer(GL_FRAMEBUFFER, 0);
        glClearColor(0.0f, 1.0f, 1.0f, 0.0f);

        glViewport(0, 0,WINDOW_WIDTH, WINDOW_HEIGHT);
        glClear( GL_COLOR_BUFFER_BIT);
        glUseProgram(texture_quad_shader_program);

        glActiveTexture(GL_TEXTURE1);
        glBindTexture(GL_TEXTURE_2D, rendered_textures[0].texture);
        glUniform1i(texture_id, 1);

        glEnableVertexAttribArray(0);
        glBindBuffer(GL_ARRAY_BUFFER, quad_vertex_buffer);
        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 0, nullptr);
        glDrawArrays(GL_TRIANGLES, 0, 6);
        glDisableVertexAttribArray(0);

        check_gl_err(nullptr);

        SDL_GL_SwapWindow(window);

        int delayTime = (int) (nextTime - SDL_GetTicks());
        if (delayTime > 0) {
            SDL_Delay((Uint32) delayTime);
        }

        nextTime += interval;
    }

    quit:
    // Cleanup VBO
    glDeleteBuffers(1, &vertexbuffer);
    glDeleteVertexArrays(1, &vertex_array);
    glDeleteProgram(shader_program);
    SDL_Quit();
    vr::VR_Shutdown();

    std::cout << "shutdown finished" << std::endl;

    return 0;
}


GLuint compile_shader(GLenum kind, const char *source) {
    GLuint shader = glCreateShader(kind);

    glShaderSource(shader, 1, &source, nullptr);
    glCompileShader(shader);


    GLint result;
    GLint info_log_len;

    glGetShaderiv(shader, GL_COMPILE_STATUS, &result);
    glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &info_log_len);
    if (info_log_len != 0) {
        std::vector<char> shader_err_msg(info_log_len);
        glGetShaderInfoLog(shader, info_log_len, nullptr, &shader_err_msg[0]);
        fprintf(stdout, "%s\n", &shader_err_msg[0]);
    }

    return shader;
}

GLuint compile_shader_program(const char *vertex_shader_src, const char *fragment_shader_src) {
    GLuint vertex_shader = compile_shader(GL_VERTEX_SHADER, vertex_shader_src);
    GLuint fragment_shader = compile_shader(GL_FRAGMENT_SHADER, fragment_shader_src);

    GLuint shader_program = glCreateProgram();
    glAttachShader(shader_program, vertex_shader);
    glAttachShader(shader_program, fragment_shader);
    glLinkProgram(shader_program);


    GLint result;
    GLint info_log_len;

    glGetProgramiv(shader_program, GL_LINK_STATUS, &result);
    glGetProgramiv(shader_program, GL_INFO_LOG_LENGTH, &info_log_len);
    if (info_log_len != 0) {
        std::vector<char> shader_err_msg(info_log_len);
        glGetShaderInfoLog(shader_program, info_log_len, nullptr, &shader_err_msg[0]);
        fprintf(stdout, "%s\n", &shader_err_msg[0]);
    }

    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);

    return shader_program;
}

void check_gl_err_impl(int line, const char *func) {
    GLenum err;
    while ((err = glGetError())) {
        std::cerr << "err #" << line;
        if (func && *func) {
            std::cerr << "(" << func << ")";
        }
        std::cerr << ": 0x" << std::hex << err << std::dec << ": " << gluErrorString(err) << std::endl;
    }
}

void handle_input_err(vr::EVRInputError error) {
    if (error != vr::VRInputError_None) {
        std::cerr << "input error: " << error << std::endl;
    }
}

void get_texture_data(GLuint texture, GLint level) {
    static int index = 0;

    GLint w, h;

    glBindTexture(GL_TEXTURE_2D, texture);
    glGetTexLevelParameteriv(GL_TEXTURE_2D, level, GL_TEXTURE_WIDTH, &w);
    glGetTexLevelParameteriv(GL_TEXTURE_2D, level, GL_TEXTURE_HEIGHT, &h);

    std::vector<uint8_t> tex_data(w * h * 4);

    glGetTexImage(GL_TEXTURE_2D, level,
                  GL_RGBA, GL_UNSIGNED_BYTE,
                  &tex_data[0]);
    check_gl_err(__func__);

    std::filesystem::create_directories("frames");

    std::stringstream bmp_path_builder;
    bmp_path_builder << "frames/frame_" << std::setfill('0') << std::setw(5) << (index++) << ".bmp";
    std::string bmp_path = bmp_path_builder.str();
    std::ofstream bmp_file;
    bmp_file.open(bmp_path, std::ios::out);

    int32_t int_slot;

#define write_int(data, size) do { int_slot = data; bmp_file.write(reinterpret_cast<const char *>(&int_slot), size);} while (0)
    // file header
    bmp_file << "BM";
    write_int((int32_t)(tex_data.size()) + 12 + 14, 4);
    write_int(0, 4);
    write_int(12 + 14, 4); // sum of header size

    // OS/2 bitmap header
    write_int(12, 4);
    write_int(w, 2);
    write_int(h, 2);
    write_int(1, 2);
    write_int(32, 2);
    bmp_file.write(reinterpret_cast<const char *>(&tex_data[0]), tex_data.size());
#undef write_int

    bmp_file.close();
}
