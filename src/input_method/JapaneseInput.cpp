//
// Created by anatawa12 on 2022/09/14.
//

#include "JapaneseInput.h"

using namespace std::string_literals;

InputNextAction JapaneseInput::onInput(glm::i8vec2 chars) {
  InputNextAction result = InputNextAction::Nop;
  switch (to64(chars.x, chars.y)) {
    case to64(4, 5):
      // add SmallChar
      processLastChar(buffer, [](char32_t c) -> char32_t {
        if (U"あいうえおつやゆよわ"s.find(c) != std::u32string::npos) {
          return (c - 1);
        } else if (U"ぁぃぅぇぉっゃゅょゎ"s.find(c) != std::u32string::npos) {
          return (c + 1);
        } else if (c == U'か') {
          return U'ゕ';
        } else if (c == U'ゕ') {
          return U'か';
        } else if (c == U'け') {
          return U'ゖ';
        } else if (c == U'ゖ') {
          return U'け';
        } else {
          return c;
        }
      });
    case to64(4, 6):
      // add Dakuten
      processLastChar(buffer, [](char32_t c) -> char32_t {
        if (U"かきくけこさしすせそたちつてとはひふへほ"s.find(c) != std::u32string::npos) {
          return (c + 1);
        } else if (U"がぎぐげござじずぜぞだぢづでどばびぶべぼ"s.find(c) != std::u32string::npos) {
          return (c - 1);
        } else if (c == U'う') {
          return U'ゔ';
        } else if (c == U'ゔ') {
          return U'う';
        } else {
          return c;
        }
      });
      break;
    case to64(4, 7):
      // add Handakuten
      processLastChar(buffer, [](char32_t c) -> char32_t {
        if (U"はひふへほ"s.find(c) != std::u32string::npos) {
          return (c + 2);
        } else if (U"ぱぴぷぺぽ"s.find(c) != std::u32string::npos) {
          return (c - 2);
        } else {
          return c;
        }
      });
      break;
    case to64(5, 5):
      // nop
      break;
    case to64(5, 6):
      if (buffer.empty()) {
        result = InputNextAction::CloseKeyboard;
      } else {
        // Henkan
      }
      break;
    case to64(5, 7):
      if (buffer.empty()) {
        result = InputNextAction::NewLine;
      } else {
        // Kakutei
        result = InputNextAction::FlushBuffer;
      }
      break;
    case to64(6, 6):
      if (!removeLastChar(buffer)) {
        result = InputNextAction::RemoveLastChar;
      }
      break;
    case to64(6, 7):
      buffer += ' ';
      break;
    case to64(7, 6):
      result = InputNextAction::MoveToSignPlane;
      break;
    case to64(7, 7):
      result = InputNextAction::MoveToNextPlane;
      break;
    default:
      buffer += table[chars.x * 8 + chars.y];
  }
  if (buffer.empty()) {
    table[to64(5, 6)] = u8"閉じる";
    table[to64(5, 7)] = ReturnSign;
  } else {
    table[to64(5, 6)] = u8"変換";
    table[to64(5, 7)] = u8"確定";
  }
  return result;
}

#define DAKUTEN_ICON u8"\u2B1A\u3099"
#define HANDAKUTEN_ICON u8"\u2B1A\u309a"

JapaneseInput::JapaneseInput() {
  table = {
      u8"あ", u8"い", u8"う", u8"え", u8"お", u8"や", u8"ゆ", u8"よ",
      u8"か", u8"き", u8"く", u8"け", u8"こ", u8"わ", u8"を", u8"ん",
      u8"さ", u8"し", u8"す", u8"せ", u8"そ", u8"「", u8"。", u8"?",
      u8"た", u8"ち", u8"つ", u8"て", u8"と", u8"」", u8"、", u8"!",
      u8"な", u8"に", u8"ぬ", u8"ね", u8"の", u8"小", DAKUTEN_ICON, HANDAKUTEN_ICON,
      u8"は", u8"ひ", u8"ふ", u8"へ", u8"ほ", u8"", u8"閉じる", ReturnSign,
      u8"ま", u8"み", u8"む", u8"め", u8"も", u8"ー", BackspaceIcon, SpaceIcon,
      u8"ら", u8"り", u8"る", u8"れ", u8"ろ", u8"〜", SignsIcon, NextPlaneIcon,
  };
}

std::u8string JapaneseInput::getAndClearBuffer() {
  auto result = AbstractInputMethod::getAndClearBuffer();
  table[to64(5, 6)] = u8"閉じる";
  table[to64(5, 7)] = ReturnSign;
  return std::move(result);
}
