cmake_minimum_required(VERSION 3.0)
add_compile_options("$<$<C_COMPILER_ID:MSVC>:/utf-8>")
add_compile_options("$<$<CXX_COMPILER_ID:MSVC>:/utf-8>")

project(clekey_ovr)

set(CMAKE_CXX_STANDARD 20)

include(cmake/openvr.cmake)
find_package(SDL2 REQUIRED)
find_package(GLEW REQUIRED)
set(OpenGL_GL_PREFERENCE "GLVND")
find_package(OpenGL REQUIRED)
find_package(Freetype REQUIRED)

add_executable(clekey_ovr
        src/main.cpp
        src/OVRController.cpp
        src/graphics/MainGuiRenderer.cpp
        src/graphics/DesktopGuiRenderer.cpp
        src/graphics/bmp_export.cpp
        src/graphics/BackgroundRingRenderer.cpp
        src/graphics/CursorCircleRenderer.cpp
        src/graphics/FreetypeRenderer.cpp
        )

configure_file(resources/actions.json actions.json COPYONLY)
configure_file(resources/bindings_oculus_touch.json bindings_oculus_touch.json COPYONLY)
configure_file(resources/bindings_vive_controller.json bindings_vive_controller.json COPYONLY)
configure_file(resources/fonts/NotoSansJP-Medium.otf fonts/NotoSansJP-Medium.otf COPYONLY)
configure_file(resources/fonts/NotoEmoji-VariableFont_wght.ttf fonts/NotoEmoji-VariableFont_wght.ttf COPYONLY)
configure_file(resources/fonts/NotoSansSymbols2-Regular.ttf fonts/NotoSansSymbols2-Regular.ttf COPYONLY)
add_definitions(-DWITH_OPEN_VR)
link_openvr(clekey_ovr)
include_directories(
        ${SDL2_INCLUDE_DIRS}
        ${OPENGL_INCLUDE_DIRS}
        oglwrap
        glm
)
target_link_libraries(
        clekey_ovr
        ${SDL2_LIBRARIES}
        GLEW::GLEW
        Freetype::Freetype
        ${OPENGL_LIBRARIES}
)

add_executable(freetype_test src/freetype_test.cpp)
target_link_libraries(freetype_test Freetype::Freetype)
