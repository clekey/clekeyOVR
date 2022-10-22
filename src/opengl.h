// wrapper of opengl.h

#if defined(_WIN32)
#include <Windows.h>
#include <GL/gl.h>
#include <GL/glu.h>
#elif defined(__APPLE__)
#include <OpenGL/gl.h>
#include <OpenGL/glu.h>
#else
#error "opengl not found"
#endif