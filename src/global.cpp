//
// Created by anatawa12 on 2022/09/17.
//

#include "global.h"

#if WIN32
#include "windows.h"

namespace {

std::filesystem::path computeExePath() {
  wchar_t path[FILENAME_MAX] = {0};
  GetModuleFileNameW(nullptr, path, FILENAME_MAX);
  return std::filesystem::path(path);
}

std::filesystem::path computeConfigDir() {
  return std::filesystem::path(_wgetenv(L"APPDATA")) / "clekey_ovr";
}

}
#elif defined(__APPLE__)
#include <mach-o/dyld.h>
#include <sys/param.h>
#include <sysdir.h>
#include <NSSystemDirectories.h>

namespace {

std::filesystem::path computeExePath() {
  char buf_stack[MAXPATHLEN] = {0};
  char* buf = buf_stack;
  uint32_t bufsize = MAXPATHLEN;
  switch (_NSGetExecutablePath(buf, &bufsize)) {
    case 0:
      // success
      break;
    case -1:
      // not enough: retry
      buf = static_cast<char *>(malloc(bufsize));
      assert(_NSGetExecutablePath(buf, &bufsize) == 0);
      break;
    default:
      abort();
  }
  return {reinterpret_cast<char8_t *>(buf)};
}

std::filesystem::path computeConfigDir() {
  auto s = sysdir_start_search_path_enumeration(SYSDIR_DIRECTORY_APPLICATION_SUPPORT, SYSDIR_DOMAIN_MASK_USER);
  char buf[PATH_MAX] = {0};
  sysdir_get_next_search_path_enumeration(s, buf);
  return std::filesystem::path(reinterpret_cast<char8_t *>(buf)) / "clekey_ovr";
}

}

#endif

std::filesystem::path getExePath() {
  static std::filesystem::path path = computeExePath();
  return path;
}

std::filesystem::path getExeDir() {
  static std::filesystem::path path = getExePath().parent_path();
  return path;
}

std::filesystem::path getResourcesDir() {
  static std::filesystem::path path = getExeDir() / "resources";
  return path;
}

std::filesystem::path getConfigDir() {
  static std::filesystem::path path = computeConfigDir();
  return path;
}
