//
// Created by anatawa12 on 8/10/22.
//

#ifndef CLEKEY_OVR_GL_UTILS_H
#define CLEKEY_OVR_GL_UTILS_H

#include <GL/glew.h>
#include <stdexcept>

#define GLNameWrapper(ClassName, GLNameType) \
private:                                             \
    explicit ClassName(GLNameType name, int dummy) noexcept: name(name) {} \
public:                                              \
    const GLNameType name;                           \
    static ClassName of_name(GLuint name) noexcept { \
        return ClassName(name, 0);                   \
    }                                                \
private:

#define GLResourceNameWrapper(ClassName, GLNameType) \
GLNameWrapper(ClassName, GLNameType)private:                                             \
public:                                              \
    ClassName(ClassName &&source) noexcept: name(source.name) {            \
        const_cast<GLNameType&>(source.name) = 0;    \
    }                                                \
                                                     \
    ClassName& operator=(ClassName&& other) noexcept {                     \
        if (this != &other) {                        \
            deallocate();                            \
            const_cast<GLNameType&>(name) = other.name;                    \
            const_cast<GLNameType&>(other.name) = 0;\
        }                                            \
        return *this;                                \
    }                                                \
                                                     \
    ~ClassName() {                                   \
        deallocate();                                \
    }                                                \
                                                     \
    ClassName(const ClassName &) = delete;           \
    ClassName &operator=(const ClassName &other) = delete;                 \
private:

class GLShader {
    GLResourceNameWrapper(GLShader, GLuint)

    void deallocate() const noexcept {
        if (name != 0)
            glDeleteShader(name);
    }

public:
    explicit GLShader(GLenum kind) noexcept: name(glCreateShader(kind)) {}

    static GLShader compile(GLenum kind, const char *source) {
        GLShader shader = GLShader(kind);

        glShaderSource(shader.name, 1, &source, nullptr);
        glCompileShader(shader.name);


        GLint result;
        GLint info_log_len;

        glGetShaderiv(shader.name, GL_COMPILE_STATUS, &result);
        glGetShaderiv(shader.name, GL_INFO_LOG_LENGTH, &info_log_len);
        if (info_log_len != 0) {
            std::vector<char> shader_err_msg(info_log_len);
            glGetShaderInfoLog(shader.name, info_log_len, nullptr, &shader_err_msg[0]);
            throw std::runtime_error(std::string(&shader_err_msg[0]));
        }

        return std::move(shader);
    }
};

class GLUniformLocation {
    GLNameWrapper(GLUniformLocation, GLint)
public:
#define UniformWrapperFn1(suffix, type) \
    void set1##suffix(type v0) const noexcept { glUniform1##suffix(name, v0); }
#define UniformWrapperFn2(suffix, type) \
    void set2##suffix(type v0, type v1) const noexcept { glUniform2##suffix(name, v0, v1); }
#define UniformWrapperFn3(suffix, type) \
    void set3##suffix(type v0, type v1, type v2) const noexcept { glUniform3##suffix(name, v0, v1, v2); }
#define UniformWrapperFn4(suffix, type) \
    void set4##suffix(type v0, type v1, type v2, type v3) const noexcept { glUniform4##suffix(name, v0, v1, v2, v3); }
#define UniformWrapperFnNv(suffix, type) \
    void set##suffix(GLsizei count, const type *value) const noexcept { glUniform##suffix(name, count, value); }
#define UniformWrapperFnMatrixNv(suffix, type) \
    void setMatrix##suffix(GLsizei count, GLboolean transpose, const type *value) const noexcept { \
        glUniformMatrix##suffix(name, count, transpose, value);                                    \
    }

    UniformWrapperFn1(f, GLfloat)
    UniformWrapperFnNv(1fv, GLfloat)
    UniformWrapperFn2(f, GLfloat)
    UniformWrapperFnNv(2fv, GLfloat)
    UniformWrapperFn3(f, GLfloat)
    UniformWrapperFnNv(3fv, GLfloat)
    UniformWrapperFn4(f, GLfloat)
    UniformWrapperFnNv(4fv, GLfloat)
    UniformWrapperFn1(i, GLint)
    UniformWrapperFnNv(1iv, GLint)
    UniformWrapperFn2(i, GLint)
    UniformWrapperFnNv(2iv, GLint)
    UniformWrapperFn3(i, GLint)
    UniformWrapperFnNv(3iv, GLint)
    UniformWrapperFn4(i, GLint)
    UniformWrapperFnNv(4iv, GLint)
    UniformWrapperFn1(ui, GLuint)
    UniformWrapperFnNv(1uiv, GLuint)
    UniformWrapperFn2(ui, GLuint)
    UniformWrapperFnNv(2uiv, GLuint)
    UniformWrapperFn3(ui, GLuint)
    UniformWrapperFnNv(3uiv, GLuint)
    UniformWrapperFn4(ui, GLuint)
    UniformWrapperFnNv(4uiv, GLuint)

    UniformWrapperFnMatrixNv(2fv, GLfloat)
    UniformWrapperFnMatrixNv(3fv, GLfloat)
    UniformWrapperFnMatrixNv(4fv, GLfloat)
    UniformWrapperFnMatrixNv(2x3fv, GLfloat)
    UniformWrapperFnMatrixNv(3x2fv, GLfloat)
    UniformWrapperFnMatrixNv(2x4fv, GLfloat)
    UniformWrapperFnMatrixNv(4x2fv, GLfloat)
    UniformWrapperFnMatrixNv(3x4fv, GLfloat)
    UniformWrapperFnMatrixNv(4x3fv, GLfloat)
};

class GLShaderProgram {
    GLResourceNameWrapper(GLShaderProgram, GLuint)

    void deallocate() const noexcept {
        if (name != 0)
            glDeleteProgram(name);
    }

public:
    GLShaderProgram() noexcept: name(glCreateProgram()) {}

    static GLShaderProgram compile(const char *vertex_shader_src, const char *fragment_shader_src) {
        GLShader vertex_shader = GLShader::compile(GL_VERTEX_SHADER, vertex_shader_src);
        GLShader fragment_shader = GLShader::compile(GL_FRAGMENT_SHADER, fragment_shader_src);


        GLShaderProgram shader_program;
        glAttachShader(shader_program.name, vertex_shader.name);
        glAttachShader(shader_program.name, fragment_shader.name);
        glLinkProgram(shader_program.name);


        GLint result;
        GLint info_log_len;

        glGetProgramiv(shader_program.name, GL_LINK_STATUS, &result);
        glGetProgramiv(shader_program.name, GL_INFO_LOG_LENGTH, &info_log_len);
        if (info_log_len != 0) {
            std::vector<char> shader_err_msg(info_log_len);
            glGetShaderInfoLog(shader_program.name, info_log_len, nullptr, &shader_err_msg[0]);
            throw std::runtime_error(std::string(&shader_err_msg[0]));
        }

        return shader_program;
    }

    GLUniformLocation getUniformLocation(const GLchar *uniform_name) const noexcept {
        return GLUniformLocation::of_name(glGetUniformLocation(name, uniform_name));
    }

    void use() const noexcept {
        return glUseProgram(name);
    }
};

#endif //CLEKEY_OVR_GL_UTILS_H
