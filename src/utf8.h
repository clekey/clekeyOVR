//
// Created by anatawa12 on 2022/09/09.
//

#ifndef CLEKEY_OVR_UTF8_H
#define CLEKEY_OVR_UTF8_H

#include <stdexcept>
#include <string>

inline char8_t check2ndByte(char8_t c) {
  if (0x80 <= c && c <= 0xBF) return c;
  throw std::runtime_error("invalid utf8 code");
}

inline char32_t checkRange(char32_t c, char32_t min, char32_t max) {
  if (min <= c && c <= max) return c;
  throw std::runtime_error("invalid utf8 code");
}

template<std::input_iterator Iterator>
inline char32_t parse_u8(const Iterator &it) {
  char8_t b1 = *it;
  if (b1 <= 0x7F) {
    return b1;
  } else if (b1 <= 0xBF) {
    // 2nd byte
    throw std::runtime_error("invalid utf8 code");
  } else if (b1 <= 0xDF) {
    // 2 bytes
    char8_t b2 = check2ndByte(*(it + 1));
    return checkRange((b1 & 0x1F) << 6 | (b2 & 0x3F), 0x0080, 0x07FF);
  } else if (b1 <= 0xEF) {
    // 3 bytes
    char8_t b2 = check2ndByte(*(it + 1));
    char8_t b3 = check2ndByte(*(it + 2));
    return checkRange((b1 & 0x0F) << 12 | (b2 & 0x3F) << 6 | (b3 & 0x3F), 0x0800, 0xFFFF);
  } else if (b1 <= 0xF7) {
    // 4 bytes
    char8_t b2 = check2ndByte(*(it + 1));
    char8_t b3 = check2ndByte(*(it + 2));
    char8_t b4 = check2ndByte(*(it + 3));
    return checkRange((b1 & 0x07) << 18 | (b2 & 0x3F) << 12 | (b3 & 0x3F) << 6 | (b4 & 0x3F), 0x10000, 0x10FFFF);
  } else {
    // 5bytes or more: invalid utf8
    throw std::runtime_error("invalid utf8 code");
  }
}

template<std::input_iterator Iterator>
inline int increment_u8(const Iterator &it) {
  char8_t b1 = *it;
  if (b1 <= 0x7F) {
    return 1;
  } else if (b1 <= 0xBF) {
    // 2nd byte
    throw std::runtime_error("invalid utf8 code");
  } else if (b1 <= 0xDF) {
    // 2 bytes
    return 2;
  } else if (b1 <= 0xEF) {
    // 3 bytes
    return 3;
  } else if (b1 <= 0xF7) {
    // 4 bytes
    return 4;
  } else {
    // 5bytes or more: invalid utf8
    throw std::runtime_error("invalid utf8 code");
  }
}

template<std::input_iterator Iterator>
inline int decrement_u8(const Iterator &it) {
  int decrement = 1;
  while ((*(it - decrement) & 0xC0) == 0x80)
    decrement++;
  return decrement;
}

template<std::random_access_iterator Iterator, std::enable_if_t<std::is_same<char8_t, typename std::iterator_traits<Iterator>::value_type>::value, std::nullptr_t> = nullptr>
class u8u32iterator {
private:
  using ref_iterator_type = Iterator;
  ref_iterator_type it_;

public:
  u8u32iterator() = default;

  explicit u8u32iterator(Iterator it) noexcept: it_(it) {}

  u8u32iterator(const u8u32iterator &) noexcept = default;

  u8u32iterator(u8u32iterator &&) noexcept = default;

  u8u32iterator &operator=(const u8u32iterator &) noexcept = default;

  u8u32iterator &operator=(u8u32iterator &&) noexcept = default;

  bool operator==(u8u32iterator &other) noexcept {
    return it_ == other.it_;
  }

  [[nodiscard]] ref_iterator_type get_raw_iterator() const { return it_; }

  using iterator_category = typename std::iterator_traits<Iterator>::iterator_category;
  using value_type = char32_t;
  using difference_type = std::ptrdiff_t;
  using pointer = char32_t *;
  using reference = char32_t &;

  char32_t operator*() const noexcept {
    return parse_u8(it_);
  }

  u8u32iterator &operator++() noexcept {
    it_ += increment_u8(it_);
    return *this;
  }

  u8u32iterator &operator--() noexcept {
    it_ -= decrement_u8(it_);
    return *this;
  }
};

template<typename It>
class u8u32range {
public:
  using iterator = u8u32iterator<It>;
private:
  iterator begin_;
  iterator end_;
public:
  u8u32range() = delete;

  u8u32range(It begin, It end) : begin_(begin), end_(end) {}

  u8u32range(const u8u32range &) = default;

  u8u32range(u8u32range &&) noexcept = default;

  u8u32range &operator=(const u8u32range &) = default;

  u8u32range &operator=(u8u32range &&) noexcept = default;

  iterator &begin() noexcept { return this->begin_; }

  [[nodiscard]] const iterator &begin() const noexcept { return this->begin_; }

  iterator &end() noexcept { return this->end_; }

  [[nodiscard]] const iterator &end() const noexcept { return this->end_; }
};

template<typename It, std::enable_if_t<std::is_same<char8_t, typename std::iterator_traits<It>::value_type>::value, std::nullptr_t> = nullptr>
inline u8u32range<It> make_u8u32range(It begin, It end) {
  return {begin, end};
}

template<typename Container>
inline u8u32range<typename Container::const_iterator> make_u8u32range(const Container &c) {
  return u8u32range(c.begin(), c.end());
}

template<std::size_t N>
inline u8u32range<char8_t *> make_u8u32range(char8_t (&arr)[N]) {
  return u8u32range(std::begin(arr), std::end(arr));
}

// utf32 -> utf8

inline std::u8string toUTF8(char32_t utf32) {
  if (utf32 < 0x80) {
    return {(char8_t) utf32};
  } else if (utf32 < 0x800) {
    char8_t first = 0xC0 | ((utf32 >> 6) & 0x1F);
    char8_t second = 0x80 | ((utf32 >> 0) & 0x3F);
    return {first, second};
  } else if (utf32 < 0x10000) {
    char8_t first = 0xE0 | ((utf32 >> 12) & 0x0F);
    char8_t second = 0x80 | ((utf32 >> 6) & 0x3F);
    char8_t third = 0x80 | ((utf32 >> 0) & 0x3F);
    return {first, second, third};
  } else if (utf32 < 0x10FFFF) {
    char8_t first = 0xF0 | ((utf32 >> 18) & 0x07);
    char8_t second = 0x80 | ((utf32 >> 12) & 0x3F);
    char8_t third = 0x80 | ((utf32 >> 6) & 0x3F);
    char8_t fourth = 0x80 | ((utf32 >> 0) & 0x3F);
    return {first, second, third, fourth};
  } else {
    throw std::runtime_error("invalid utf32 code");
  }
}

#endif //CLEKEY_OVR_UTF8_H
