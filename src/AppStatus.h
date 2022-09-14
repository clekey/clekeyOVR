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

struct HandInfo {
  glm::vec2 stick;
  int8_t selection;

  bool clicking: 1;
  bool clickingOld: 1;

  bool startClick() const {
    return clicking && !clickingOld;
  }
};

struct AppStatus {
  HandInfo left;
  HandInfo right;

  std::array<std::u8string, 8 * 8> chars;

  [[nodiscard]] const HandInfo& getControllerInfo(LeftRight side) const {
    return side == LeftRight::Left ? left : right;
  }

  [[nodiscard]] HandInfo& getControllerInfo(LeftRight side) {
    return side == LeftRight::Left ? left : right;
  }

  [[nodiscard]] glm::vec2 getStickPos(LeftRight side) const {
    return side == LeftRight::Left ? left.stick : right.stick;
  }

  [[nodiscard]] int8_t getSelectingOfCurrentSide(LeftRight side) const {
    return side == LeftRight::Left ? left.selection : right.selection;
  }

  [[nodiscard]] int8_t getSelectingOfOppositeSide(LeftRight side) const {
    return side == LeftRight::Left ? right.selection : left.selection;
  }
};

#endif //CLEKEY_OVR_APPSTATUS_H
