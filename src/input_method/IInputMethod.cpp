//
// Created by anatawa12 on 2022/09/14.
//

#include "IInputMethod.h"

const std::array<std::u8string, 8 * 8> &AbstractInputMethod::getTable() const {
  return table;
}

const std::u8string &AbstractInputMethod::getBuffer() const {
  return buffer;
}

std::u8string AbstractInputMethod::getAndClearBuffer() {
  auto r = std::move(buffer);
  buffer = u8"";
  return std::move(r);
}

AbstractInputMethod::~AbstractInputMethod() = default;
