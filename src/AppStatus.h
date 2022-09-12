//
// Created by anatawa12 on 2022/09/12.
//

#ifndef CLEKEY_OVR_APPSTATUS_H
#define CLEKEY_OVR_APPSTATUS_H

#include "glm/glm.hpp"
#include <string>
#include <array>

enum LeftRight {
  Left,
  Right,
};

struct AppStatus {
  glm::vec2 leftStickPos;
  glm::vec2 rightStickPos;
  int8_t leftSelection;
  int8_t rightSelection;

  std::array<std::u8string, 8 * 8> chars;

  [[nodiscard]] glm::vec2 getStickPos(LeftRight side) const {
    return side == LeftRight::Left ? leftStickPos : rightStickPos;
  }

  [[nodiscard]] int8_t getSelectingOfCurrentSide(LeftRight side) const {
    return side == LeftRight::Left ? leftSelection : rightSelection;
  }

  [[nodiscard]] int8_t getSelectingOfOppositeSide(LeftRight side) const {
    return side == LeftRight::Left ? rightSelection : leftSelection;
  }
};

#endif //CLEKEY_OVR_APPSTATUS_H
