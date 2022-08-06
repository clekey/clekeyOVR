cmake_minimum_required(VERSION 3.0)
project(clekey_ovr)

set(CMAKE_CXX_STANDARD 14)

include(cmake/openvr.cmake)

add_executable(clekey_ovr src/main.cpp)
link_openvr(clekey_ovr)
