//
// Created by anatawa12 on 2022/09/14.
//

#ifndef CLEKEY_OVR_IINPUTMETHOD_H
#define CLEKEY_OVR_IINPUTMETHOD_H

#include <array>
#include <string>
#include "glm/glm.hpp"

#include "../utf8.h"
#include "HardKeyButton.h"
#include <bitset>
#include <concepts>

#define to64(x, y) ((x) * 8 + (y))

inline char32_t lastChar(const std::u8string &str) {
  auto iter = make_u8u32range(str).end();
  return *(--iter);
}

template<std::regular_invocable<char32_t> CharReplacer>
inline void processLastChar(std::u8string &buffer, CharReplacer charReplacer) {
  if (buffer.length() == 0) return;

  char32_t c = lastChar(buffer);
  int dec = decrement_u8(buffer.end());
  buffer.resize(buffer.length() - dec);
  c = charReplacer(c);
  buffer += toUTF8(c);
}

inline bool removeLastChar(std::u8string &buffer) {
  if (buffer.length() == 0) return false;

  char32_t c = lastChar(buffer);
  int dec = decrement_u8(buffer.end());
  buffer.resize(buffer.length() - dec);
  return true;
}

const std::u8string BackspaceIcon = u8"⌫";
const std::u8string SpaceIcon = u8"␣";
const std::u8string NextPlaneIcon = u8"\U0001F310"; // 🌐
const std::u8string SignsIcon = u8"#+=";
const std::u8string ReturnSign = u8"⏎";

enum class InputNextAction {
  Nop,
  MoveToNextPlane,
  // this will be used to back to char plane in sign plane
  MoveToSignPlane,
  FlushBuffer,
  RemoveLastChar,
  CloseKeyboard,
  NewLine,
};

class IInputMethod {
public:
  [[nodiscard]] virtual const std::array<std::u8string, 8 * 8> &getTable() const = 0;

  [[nodiscard]] virtual const std::u8string &getBuffer() const = 0;

  [[nodiscard]] virtual std::u8string getAndClearBuffer() = 0;

  virtual InputNextAction onInput(glm::i8vec2) = 0;

  virtual InputNextAction onHardInput(HardKeyButton) = 0;

  virtual ~IInputMethod() = default;
};

class AbstractInputMethod : public IInputMethod {
protected:
  std::array<std::u8string, 8 * 8> table;
  std::u8string buffer;
public:
  AbstractInputMethod() : table{} {}

  [[nodiscard]] const std::array<std::u8string, 8 * 8> &getTable() const override;

  [[nodiscard]] const std::u8string &getBuffer() const override;

  [[nodiscard]] std::u8string getAndClearBuffer() override;

  ~AbstractInputMethod() override;
};

#endif //CLEKEY_OVR_IINPUTMETHOD_H
