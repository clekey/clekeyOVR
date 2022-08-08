#include <iostream>
#include "openvr.h"
#include <SDL.h>
#include <GL/glew.h>
#include <vector>

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

#define check_gl_err() check_gl_err_impl(__LINE__)
void check_gl_err_impl(int line);
void handle_input_err(vr::EVRInputError error);

vr::VRActionHandle_t action_left_stick;
vr::VRActionHandle_t action_left_click;
vr::VRActionHandle_t action_left_haptic;
vr::VRActionHandle_t action_right_stick;
vr::VRActionHandle_t action_right_click;
vr::VRActionHandle_t action_right_haptic;
vr::VRActionSetHandle_t action_set_input;

int main(int argc, char** argv) {
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

    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 2);
    SDL_GLContext context = SDL_GL_CreateContext(window);
    if (!context) return -1;

    glewExperimental = true;
    glewInit();

    GLuint vertex_array;
    glGenVertexArrays(1, &vertex_array);
    glBindVertexArray(vertex_array);

    GLuint shader_program;
    {
        GLuint vertex_shader = glCreateShader(GL_VERTEX_SHADER);
        GLuint fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
        const char *vertex_shader_src =
                "#version 330 core\n"
                "layout(location = 0) in vec3 vertexPosition_modelspace;\n"
                "void main() {\n"
                "    gl_Position.xyz = vertexPosition_modelspace;\n"
                "}\n";
        const char *fragment_shader_src =
                "#version 330 core\n"
                "// Ouput data\n"
                "out vec3 color;\n"
                "\n"
                "void main() {\n"
                "    // Output color = red \n"
                "    color = vec3(1,0,0);\n"
                "}\n";

        glShaderSource(vertex_shader, 1, &vertex_shader_src, nullptr);
        glCompileShader(vertex_shader);

        {
            GLint result;
            GLint info_log_len;

            glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &result);
            glGetShaderiv(vertex_shader, GL_INFO_LOG_LENGTH, &info_log_len);
            if (info_log_len != 0) {
                std::vector<char> shader_err_msg(info_log_len);
                glGetShaderInfoLog(vertex_shader, info_log_len, nullptr, &shader_err_msg[0]);
                fprintf(stdout, "%s\n", &shader_err_msg[0]);
            }
        }

        glShaderSource(fragment_shader, 1, &fragment_shader_src, nullptr);
        glCompileShader(fragment_shader);

        {
            GLint result;
            GLint info_log_len;

            glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &result);
            glGetShaderiv(fragment_shader, GL_INFO_LOG_LENGTH, &info_log_len);
            if (info_log_len != 0) {
                std::vector<char> shader_err_msg(info_log_len);
                glGetShaderInfoLog(fragment_shader, info_log_len, nullptr, &shader_err_msg[0]);
                fprintf(stdout, "%s\n", &shader_err_msg[0]);
            }
        }

        shader_program = glCreateProgram();
        glAttachShader(shader_program, vertex_shader);
        glAttachShader(shader_program, fragment_shader);
        glLinkProgram(shader_program);

        {
            GLint result;
            GLint info_log_len;

            glGetProgramiv(shader_program, GL_LINK_STATUS, &result);
            glGetProgramiv(shader_program, GL_INFO_LOG_LENGTH, &info_log_len);
            if (info_log_len != 0) {
                std::vector<char> shader_err_msg(info_log_len);
                glGetShaderInfoLog(shader_program, info_log_len, nullptr, &shader_err_msg[0]);
                fprintf(stdout, "%s\n", &shader_err_msg[0]);
            }
        }

        glDeleteShader(vertex_shader);
        glDeleteShader(fragment_shader);
    }

    // setup viewport
    glViewport(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
    glClearColor(0.0f, 0.0f, 0.0f, 0.0f);

    static const GLfloat g_vertex_buffer_data[] = {
            -1.0f, -1.0f, 0.0f,
            1.0f, -1.0f, 0.0f,
            0.0f,  1.0f, 0.0f,
    };
    GLuint vertexbuffer;
    glGenBuffers(1, &vertexbuffer);
    glBindBuffer(GL_ARRAY_BUFFER, vertexbuffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof(g_vertex_buffer_data), g_vertex_buffer_data, GL_STATIC_DRAW);

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
        handle_input_err(vr::VRInput()->GetAnalogActionData(action_left_stick, &analog_data, sizeof (analog_data), vr::k_ulInvalidInputValueHandle));
        std::cout << "left input:  " << analog_data.bActive << ": " << analog_data.x << ", " << analog_data.y << std::endl;
        handle_input_err(vr::VRInput()->GetAnalogActionData(action_right_stick, &analog_data, sizeof (analog_data), vr::k_ulInvalidInputValueHandle));
        std::cout << "right input: " << analog_data.bActive << ": " << analog_data.x << ", " << analog_data.y << std::endl;
        vr::InputDigitalActionData_t digital_data = {};
        handle_input_err(vr::VRInput()->GetDigitalActionData(action_left_click, &digital_data, sizeof (digital_data), vr::k_ulInvalidInputValueHandle));
        std::cout << "left click:  " << digital_data.bActive << ": " << digital_data.bState << std::endl;
        handle_input_err(vr::VRInput()->GetDigitalActionData(action_right_click, &digital_data, sizeof (digital_data), vr::k_ulInvalidInputValueHandle));
        std::cout << "right click: " << digital_data.bActive << ": " << digital_data.bState << std::endl;

        SDL_Event ev;
        SDL_Keycode key;
        while ( SDL_PollEvent(&ev) )
        {
            switch(ev.type){
                case SDL_QUIT:
                    goto quit;
                case SDL_KEYDOWN:
                    key = ev.key.keysym.sym;
                    if(key == SDLK_ESCAPE)
                        goto quit;
                    break;
            }
        }

        glUseProgram(shader_program);

        // 1rst attribute buffer : vertices
        glEnableVertexAttribArray(0);
        glBindBuffer(GL_ARRAY_BUFFER, vertexbuffer);
        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 0, nullptr);
        // Draw the triangle !
        glDrawArrays(GL_TRIANGLES, 0, 3); // 3 indices starting at 0 -> 1 triangle
        glDisableVertexAttribArray(0);

        check_gl_err();

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

void check_gl_err_impl(int line) {
    GLenum err;
    while (err = glGetError()) {
        std::cerr << "err #" << line << ": " << gluErrorString(err) << std::endl;
    }
}

void handle_input_err(vr::EVRInputError error) {
    if (error != vr::VRInputError_None) {
        std::cerr << "input error: " << error << std::endl;
    }
}
