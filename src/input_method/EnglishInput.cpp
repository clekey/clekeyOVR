//
// Created by anatawa12 on 2022/09/14.
//

#include "EnglishInput.h"

using namespace std::string_literals;

InputNextAction EnglishInput::onInput(glm::i8vec2 chars) {
  switch (to64(chars.x, chars.y)) {
    case to64(6, 5):
      return InputNextAction::CloseKeyboard;
    case to64(6, 6):
      return InputNextAction::RemoveLastChar;
    case to64(6, 7):
      buffer = ' ';
      return InputNextAction::FlushBuffer;
    case to64(7, 5):
      return InputNextAction::NewLine;
    case to64(7, 6):
      return InputNextAction::MoveToSignPlane;
    case to64(7, 7):
      return InputNextAction::MoveToNextPlane;
    default:
      buffer = table[chars.x * 8 + chars.y];
      return InputNextAction::FlushBuffer;
  }
}

#define DAKUTEN_ICON u8"\u2B1A\u3099"
#define HANDAKUTEN_ICON u8"\u2B1A\u309a"

EnglishInput::EnglishInput() {
  table = {
      u8"a", u8"b", u8"c", u8"d", u8"e", u8"f", u8"g", u8"h",
      u8"A", u8"B", u8"C", u8"D", u8"E", u8"F", u8"G", u8"H",
      u8"i", u8"j", u8"k", u8"l", u8"m", u8"n", u8"o", u8"p",
      u8"I", u8"J", u8"K", u8"L", u8"M", u8"N", u8"O", u8"P",
      u8"q", u8"r", u8"s", u8"t", u8"u", u8"v", u8"w", u8"x",
      u8"Q", u8"R", u8"S", u8"T", u8"U", u8"V", u8"W", u8"X",
      u8"y", u8"z", u8"\"", u8".", u8"?", u8"Close", BackspaceIcon, SpaceIcon,
      u8"Y", u8"Z", u8"\'", u8",", u8"!", ReturnSign, SignsIcon, NextPlaneIcon,
  };
}
