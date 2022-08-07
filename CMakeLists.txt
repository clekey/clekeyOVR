cmake_minimum_required(VERSION 3.0)
project(clekey_ovr)

set(CMAKE_CXX_STANDARD 14)

include(cmake/openvr.cmake)
find_package(SDL2 REQUIRED)
find_package(GLEW REQUIRED)
set(OpenGL_GL_PREFERENCE "GLVND")
find_package(OpenGL REQUIRED)

add_executable(clekey_ovr src/main.cpp)

configure_file(resources/actions.json actions.json COPYONLY)
configure_file(resources/bindings_oculus_touch.json bindings_oculus_touch.json COPYONLY)
configure_file(resources/bindings_vive_controller.json bindings_vive_controller.json COPYONLY)
link_openvr(clekey_ovr)
include_directories(
        ${SDL2_INCLUDE_DIRS}
        ${GLEW_INCLUDE_DIRS}
        ${OPENGL_INCLUDE_DIRS}
)
target_link_libraries(
        clekey_ovr
        ${SDL2_LIBRARIES}
        ${GLEW_LIBRARIES}
        ${OPENGL_LIBRARIES}
)
