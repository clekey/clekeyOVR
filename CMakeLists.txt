cmake_minimum_required(VERSION 3.0)
add_compile_options("$<$<C_COMPILER_ID:MSVC>:/utf-8>")
add_compile_options("$<$<CXX_COMPILER_ID:MSVC>:/utf-8>")

project(clekey_ovr)

set(CMAKE_CXX_STANDARD 20)

include(cmake/openvr.cmake)
find_package(SDL2 REQUIRED)
set(GLEW_USE_STATIC_LIBS "TRUE")
find_package(GLEW REQUIRED)
set(OpenGL_GL_PREFERENCE "GLVND")
find_package(OpenGL REQUIRED)
find_package(Freetype REQUIRED)

add_executable(clekey_ovr
        src/main.cpp
        src/global.cpp
        src/OVRController.cpp
        src/graphics/MainGuiRenderer.cpp
        src/graphics/DesktopGuiRenderer.cpp
        src/graphics/bmp_export.cpp
        src/graphics/BackgroundRingRenderer.cpp
        src/graphics/CursorCircleRenderer.cpp
        src/graphics/FreetypeRenderer.cpp
        src/input_method/IInputMethod.cpp
        src/input_method/JapaneseInput.cpp
        src/input_method/SignsInput.cpp
        src/input_method/EnglishInput.cpp
        )

add_custom_command(TARGET clekey_ovr
        POST_BUILD COMMAND ${CMAKE_COMMAND} -E copy_directory
            ${CMAKE_SOURCE_DIR}/resources
            $<TARGET_FILE_DIR:clekey_ovr>/resources)
add_definitions(-DWITH_OPEN_VR)
add_definitions(-DGLEW_STATIC)
add_definitions(-DGL_SILENCE_DEPRECATION)
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
        GLEW::glew_s
        Freetype::Freetype
        ${OPENGL_LIBRARIES}
)

install(TARGETS clekey_ovr RUNTIME DESTINATION .)
install(DIRECTORY ${CMAKE_SOURCE_DIR}/resources DESTINATION .)
# freetype
list(GET FREETYPE_LIBRARIES 0 FREETYPE_LIBRARY)
get_filename_component(FREETYPE_DIRNAME ${FREETYPE_LIBRARY} DIRECTORY)
install(FILES ${FREETYPE_DIRNAME}/freetype.dll DESTINATION .)
# openvr
install(FILES ${OPENVR_SHARED_LIBRARIES} DESTINATION .)
# SDL2
list(GET SDL2_LIBRARIES 0 SDL2_LIBRARY)
get_filename_component(SDL2_DIRNAME "${SDL2_LIBRARY}" DIRECTORY)
install(FILES ${SDL2_DIRNAME}/SDL2.dll DESTINATION .)
# resources
install(FILES ${CMAKE_SOURCE_DIR}/README.md DESTINATION .)
install(FILES ${CMAKE_SOURCE_DIR}/for-releasing/README-SDL.txt DESTINATION .)
install(FILES ${CMAKE_SOURCE_DIR}/for-releasing/LICENSE.txt DESTINATION .)
# vc++ installer
file(DOWNLOAD "https://aka.ms/vs/17/release/vc_redist.x64.exe"  "${CMAKE_BINARY_DIR}/vc_redist.x64.exe")
install(FILES "${CMAKE_BINARY_DIR}/vc_redist.x64.exe" DESTINATION .)


# cpack configuration
set(CPACK_GENERATOR NSIS)
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "clekey for OpenVR")
SET(CPACK_PACKAGE_VENDOR "anatawa12")
set(CPACK_PACKAGE_VERSION_MAJOR 0)
set(CPACK_PACKAGE_VERSION_MINOR 1)
set(CPACK_PACKAGE_VERSION_PATCH 0)
set(CPACK_PACKAGE_INSTALL_DIRECTORY "clekey_ovr")
set(CPACK_PACKAGE_DESCRIPTION "clekey, an experimental VR keyboard, implementation for OpenVR.")
set(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_SOURCE_DIR}/for-releasing/LICENSE.txt")
set(CPACK_NSIS_EXTRA_INSTALL_COMMANDS "
    ExecWait '$INSTDIR\\\\vc_redist.x64.exe'
")
include(CPack)

add_executable(freetype_test src/freetype_test.cpp)
target_link_libraries(freetype_test Freetype::Freetype)
