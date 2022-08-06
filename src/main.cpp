#include <iostream>
#include "openvr.h"

#ifdef WIN32
#include <windows.h>
#define sleep(n) Sleep(n * 1000)
#else
#include <csignal>
#define Sleep(n) usleep(n * 1000)
#endif

int main() {
    std::cout << "Hello, World!" << std::endl;

    vr::HmdError err;
    vr::VR_Init(&err, vr::EVRApplicationType::VRApplication_Overlay);
    if (!vr::VROverlay()) {
        std::cerr << "error: " << vr::VR_GetVRInitErrorAsEnglishDescription(err) << std::endl;
        return -1;
    }

    std::cout << "successfully launched" << std::endl;

    sleep(10);

    vr::VR_Shutdown();

    std::cout << "shutdown finished" << std::endl;

    return 0;
}
