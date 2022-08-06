
function(link_openvr target)
    set(SIZEOF_VOIDP ${CMAKE_SIZEOF_VOID_P})
    if((NOT APPLE) AND (CMAKE_SIZEOF_VOID_P EQUAL 8))
        #64-bit not available in Mac.
        set(OPENVR_PROCESSOR_ARCH "64")
    else()
        set(OPENVR_PROCESSOR_ARCH "32")
    endif()

    # Get platform.
    if(WIN32)
        set(OPENVR_PLATFORM_NAME "win")
    elseif(UNIX AND NOT APPLE)
        if(CMAKE_SYSTEM_NAME MATCHES ".*Linux")
            set(OPENVR_PLATFORM_NAME "linux")
        endif()
    elseif(APPLE)
        if(CMAKE_SYSTEM_NAME MATCHES ".*Darwin.*" OR CMAKE_SYSTEM_NAME MATCHES ".*MacOS.*")
            set(OPENVR_PLATFORM_NAME "osx")
        endif()
    endif()

    if(${CMAKE_SYSTEM_PROCESSOR} MATCHES "arm")
        set(OPENVR_ARCH_NAME "arm")
    else()
        set(OPENVR_ARCH_NAME "")
    endif()

    set(OPENVR_LIBRARY_DIR "openvr/lib/${OPENVR_PLATFORM_NAME}${OPENVR_ARCH_NAME}${OPENVR_PROCESSOR_ARCH}")
    target_link_directories(${target} PUBLIC ${OPENVR_LIBRARY_DIR})

    if(${OPENVR_PLATFORM_NAME} STREQUAL "win")
        target_link_libraries(${target} openvr_api)
    else()
        target_link_libraries(${target} openvr_api)
    endif()

    target_include_directories(${target} PRIVATE "openvr/headers")
endfunction()
