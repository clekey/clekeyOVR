//
// Created by anatawa12 on 2022/09/03.
//

#ifndef CLEKEY_OVR_FREETYPE_H
#define CLEKEY_OVR_FREETYPE_H

#include <exception>
#include "ft2build.h"
#include FT_FREETYPE_H

namespace freetype {

class error : public std::exception {
public:
  FT_Error code;

  explicit error(FT_Error error) noexcept : code(error) {}

  [[nodiscard]] const char * what() const noexcept override {
    return FT_Error_String(code);
  }
};

inline void error_check(FT_Error err) {
  if (err != 0)
    throw error(err);
}

#define ftcall(func) error_check(FT_##func)

class Face;

class Freetype {
public:
  Freetype() {
    ftcall(Init_FreeType(&library));
  }

  ~Freetype() {
    ftcall(Done_FreeType(library));
  }

  Face new_face(const char *name, FT_Long face_index);

private:
  FT_Library library;
};

class Face {
  FT_Face face;
public:
  explicit Face(FT_Face face) : face(face) {}

  void setPixelSizes(FT_UInt pixel_width, FT_UInt pixel_height) {
    ftcall(Set_Pixel_Sizes(face, pixel_width, pixel_height));
  }

  FT_UInt getCharIndex(FT_ULong charcode) {
    return FT_Get_Char_Index(face, charcode);
  }

  void loadGlyph(FT_UInt index, FT_Int32 flags) {
    ftcall(Load_Glyph(face, index, flags));
  }

  void loadChar(FT_ULong charcode, FT_Int32 flags) {
    ftcall(Load_Char(face, charcode, flags));
  }

  void renderGlyph(FT_Render_Mode render_mode) {
    ftcall(Render_Glyph(face->glyph, render_mode));
  }

  FT_FaceRec_& operator *() const {
    return *face;
  }

  FT_Face operator ->() const {
    return face;
  }

  ~Face() {
    if (face) FT_Done_Face(face);
  }

  Face(Face &&src) noexcept {
    face = src.face;
    src.face = nullptr;
  }

  Face &operator=(Face &&src) noexcept {
    if (this != &src) {
      face = src.face;
      src.face = nullptr;
    }
    return *this;
  }

  Face(const Face &) = delete;

  Face &operator=(const Face &) = delete;
};

inline Face Freetype::new_face(const char *name, FT_Long face_index) {
  FT_Face face;
  ftcall(New_Face(library, name, face_index, &face));
  return Face(face);
}

}

#endif //CLEKEY_OVR_FREETYPE_H
