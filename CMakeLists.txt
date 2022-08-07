cmake_minimum_required(VERSION 3.0)
project(clekey_ovr)

set(CMAKE_CXX_STANDARD 14)

include(cmake/openvr.cmake)

add_executable(clekey_ovr src/main.cpp)

configure_file(resources/actions.json actions.json COPYONLY)
configure_file(resources/bindings_oculus_touch.json bindings_oculus_touch.json COPYONLY)
configure_file(resources/bindings_vive_controller.json bindings_vive_controller.json COPYONLY)
link_openvr(clekey_ovr)
