//
// Created by anatawa12 on 2022/09/17.
//

#include "global.h"
#include "windows.h"

namespace {

std::filesystem::path computeExePath() {
  wchar_t path[FILENAME_MAX] = {0};
  GetModuleFileNameW(nullptr, path, FILENAME_MAX);
  return std::filesystem::path(path);
}

}

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
