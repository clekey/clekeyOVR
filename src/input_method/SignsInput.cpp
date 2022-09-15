//
// Created by anatawa12 on 2022/09/14.
//

#include "SignsInput.h"

using namespace std::string_literals;

InputNextAction SignsInput::onInput(glm::i8vec2 chars) {
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
      if (chars.x < 4 || (chars.x < 6 && chars.y < 5)) {
        buffer = table[chars.x * 8 + chars.y];
        return InputNextAction::FlushBuffer;
      } else {
        return InputNextAction::Nop;
      }
  }
}

#define DAKUTEN_ICON u8"\u2B1A\u3099"
#define HANDAKUTEN_ICON u8"\u2B1A\u309a"

SignsInput::SignsInput() {
  table = {
      u8"(", u8"[", u8"{", u8"<", u8"/", u8";", u8"-", u8"_",
      u8")", u8"]", u8"}", u8">", u8"\\", u8":", u8"+", u8"=",
      u8"“", u8".", u8"?", u8"1", u8"2", u8"3", u8"4", u8"5",
      u8"‘", u8",", u8"!", u8"6", u8"7", u8"8", u8"9", u8"0",
      u8"&", u8"*", u8"¥", u8"^", u8"%", u8"", u8"", u8"",
      u8"~", u8"`", u8"@", u8"$", u8"|", u8"", u8"", u8"",
      u8"", u8"", u8"", u8"", u8"", u8"Close", BackspaceIcon, SpaceIcon,
      u8"", u8"", u8"", u8"", u8"", ReturnSign, SignsIcon, NextPlaneIcon,
  };
}
