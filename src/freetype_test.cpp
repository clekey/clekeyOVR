//
// Created by anatawa12 on 2022/09/04.
//

#include "Freetype.h"
#include <vector>
#include <filesystem>
#include <sstream>
#include <fstream>
#include <iostream>

void export_as_bmp(uint32_t width, uint32_t height, uint8_t data[]) {
  const size_t header_size = 14 + 40;
  const size_t palette_size = 4 * 256;

  static int index = 0;

  const bool is_width_padded = (width & 3) != 0;
  const uint32_t width_padded = is_width_padded ? (width & ~3) + 4 : width;

  std::vector<uint8_t> bmp_data(header_size + palette_size + width_padded * height);

  // TODO write image data

  // file header
  bmp_data[0] = 'B';
  bmp_data[1] = 'M';
  // bfSize
  bmp_data[2] = (bmp_data.size() >> 0) & 0xFF;
  bmp_data[3] = (bmp_data.size() >> 8) & 0xFF;
  bmp_data[4] = (bmp_data.size() >> 16) & 0xFF;
  bmp_data[5] = (bmp_data.size() >> 24) & 0xFF;
  // reserved
  bmp_data[6] = 0;
  bmp_data[7] = 0;
  bmp_data[8] = 0;
  bmp_data[9] = 0;
  // bfOffBits
  bmp_data[10] = ((header_size + palette_size) >> 0) & 0xFF;
  bmp_data[11] = ((header_size + palette_size) >> 8) & 0xFF;
  bmp_data[12] = ((header_size + palette_size) >> 16) & 0xFF;
  bmp_data[13] = ((header_size + palette_size) >> 24) & 0xFF;

  // OS/2 bitmap header
  bmp_data[14] = 40;
  bmp_data[15] = 0;
  bmp_data[16] = 0;
  bmp_data[17] = 0;
  // width
  bmp_data[18] = (width >> 0) & 0xFF;
  bmp_data[19] = (width >> 8) & 0xFF;
  bmp_data[20] = (width >> 16) & 0xFF;
  bmp_data[21] = (width >> 24) & 0xFF;
  // height
  bmp_data[22] = (height >> 0) & 0xFF;
  bmp_data[23] = (height >> 8) & 0xFF;
  bmp_data[24] = (height >> 16) & 0xFF;
  bmp_data[25] = (height >> 24) & 0xFF;
  // planes = 1
  bmp_data[26] = 1;
  bmp_data[27] = 0;
  // bit per pixel = 8
  bmp_data[28] = 8;
  bmp_data[29] = 0;
  // compression = 0: uncompressed
  bmp_data[30] = 0;
  bmp_data[31] = 0;
  bmp_data[32] = 0;
  bmp_data[33] = 0;
  // image size
  bmp_data[34] = 0;
  bmp_data[35] = 0;
  bmp_data[36] = 0;
  bmp_data[37] = 0;
  // x pics per meter
  bmp_data[38] = 0;
  bmp_data[39] = 0;
  bmp_data[40] = 0;
  bmp_data[41] = 0;
  // y pics per meter
  bmp_data[42] = 0;
  bmp_data[43] = 0;
  bmp_data[44] = 0;
  bmp_data[45] = 0;
  // color palette used
  bmp_data[46] = 0;
  bmp_data[47] = 0;
  bmp_data[48] = 0;
  bmp_data[49] = 0;
  // color palette important
  bmp_data[50] = 0;
  bmp_data[51] = 0;
  bmp_data[52] = 0;
  bmp_data[53] = 0;

  // color palette
  for (int i = 0; i < 256; ++i) {
    bmp_data[header_size + i * 4 + 0] = i;
    bmp_data[header_size + i * 4 + 1] = i;
    bmp_data[header_size + i * 4 + 2] = i;
    bmp_data[header_size + i * 4 + 3] = 0;
  }

  if (is_width_padded) {
    // copy per row
    for (int y = 0; y < height; ++y) {
      std::copy(data + y * width, data + (y + 1) * width, &bmp_data[header_size + palette_size + y * width_padded]);
    }
  } else {
    std::copy(data, data + (width * height), &bmp_data[header_size + palette_size]);
  }

  std::filesystem::create_directories("frames");

  std::stringstream bmp_path_builder;
  bmp_path_builder << "frames/frame_" << std::setfill('0') << std::setw(5) << (index++) << ".bmp";
  std::string bmp_path = bmp_path_builder.str();
  std::ofstream bmp_file;
  bmp_file.open(bmp_path, std::ios::out | std::ios::binary);

  bmp_file.write(reinterpret_cast<const char *>(&bmp_data[0]), (std::streamsize) bmp_data.size());

  bmp_file.close();
}

class F26Dot6 {
public:
  FT_F26Dot6 value;

  explicit F26Dot6(FT_F26Dot6 value) : value(value) {}
};

std::ostream &operator<<(std::ostream &os, const F26Dot6 &value) {
  os << (value.value >> 6) << '.';
  int rest = (int) (value.value & 0x3F);
  do {
    rest *= 10;
    os << (rest >> 6);
    rest &= 0x3F;
  } while (rest != 0);
  return os;
}

class Renderer {
  freetype::Face face;
  int cursor;
public:
  int baseline;
  int width;
  int height;
  std::vector<uint8_t> data;

  Renderer(freetype::Face face, int width, int height) : face(std::move(face)), width(width), height(height),
                                                         data(height * width) {
    cursor = 0;
    baseline = height / 2;

    for (int x = 0; x < width; ++x) {
      draw_dot(x, baseline, 127);
    }
  }

  void draw_dot(int x, int y, uint8_t color) {
    if (color == 255) {
      data[y * width + x] = 255;
      return; // fast path
    } else if (color == 0) {
      return; // fast path
    } else {
      float dst = ((float) data[y * width + x]) / 255.0f;
      float src = ((float) color) / 255.0f;
      data[y * width + x] = (int) ((dst * (1 - src) + src) * 255);
    }
  }

  void draw_string(const char32_t *c) {
    while (*c != U'\0') {
      draw_char(*c++);
    }
  }

  void draw_char(char32_t c) {
    face.setPixelSizes(baseline, 0);
    face.loadChar(c, FT_LOAD_RENDER);
    auto glyph = face->glyph;
    const auto pitchAbs = abs(glyph->bitmap.pitch);

    int xMin = (int) (glyph->metrics.horiBearingX >> 6);
    int yMin = (int) ((glyph->metrics.horiBearingY >> 6) - (glyph->metrics.height >> 6));
    int xMinPos = xMin + cursor;
    int yMinPos = yMin + baseline;

    if (glyph->bitmap.pitch < 0) {
      // glyph->bitmap.buffer is down to top: keep y
      for (int fontY = 0, dataY = yMinPos; fontY < glyph->bitmap.rows; ++fontY, ++dataY) {
        for (int fontX = 0, dataX = xMinPos; fontX < glyph->bitmap.width; ++fontX, ++dataX) {
          draw_dot(dataX, dataY, glyph->bitmap.buffer[fontY * pitchAbs + fontX]);
        }
      }
    } else {
      // glyph->bitmap.buffer is top to down: inverse y
      for (int fontY = (int) (glyph->bitmap.rows - 1), dataY = yMinPos; fontY >= 0; --fontY, ++dataY) {
        for (int fontX = 0, dataX = xMinPos; fontX < glyph->bitmap.width; ++fontX, ++dataX) {
          draw_dot(dataX, dataY, glyph->bitmap.buffer[fontY * pitchAbs + fontX]);
        }
      }
    }

    cursor += (int) (glyph->metrics.horiAdvance >> 6);
    // addtional: show origin point

    draw_dot(cursor - 1, baseline, 255);
    draw_dot(cursor + 1, baseline, 255);
    draw_dot(cursor, baseline - 1, 255);
    draw_dot(cursor, baseline + 1, 255);
    draw_dot(cursor, baseline, 255);
  }
};

int main(int argc, char **argv) {
  freetype::Freetype ft;
  auto face = ft.new_face("./fonts/NotoSansJP-Medium.otf", 0);
  auto char_size = 64;
  face.setPixelSizes(char_size, 0);
  face.loadChar(U'あ', FT_LOAD_RENDER);
  auto glyph = face->glyph;
  std::vector<uint8_t> bmp(glyph->bitmap.width * glyph->bitmap.rows);
  const auto pitchAbs = abs(glyph->bitmap.pitch);

  if (glyph->bitmap.pitch < 0) {
    for (int y = 0; y < glyph->bitmap.rows; ++y) {
      std::copy(
          &glyph->bitmap.buffer[y * pitchAbs],
          &glyph->bitmap.buffer[(y + 1) * pitchAbs],
          &bmp[y * glyph->bitmap.width]);
    }
  } else {
    for (int y = 0; y < glyph->bitmap.rows; ++y) {
      std::copy(
          &glyph->bitmap.buffer[y * pitchAbs],
          &glyph->bitmap.buffer[(y + 1) * pitchAbs],
          &bmp[(glyph->bitmap.rows - y - 1) * glyph->bitmap.width]);
    }
  }

  //export_as_bmp(glyph->bitmap.width, glyph->bitmap.rows, &bmp[0]);

  std::cout << "units_per_EM: " << face->units_per_EM << std::endl;
  std::cout << "ascender:     " << face->ascender * char_size / face->units_per_EM << std::endl;
  std::cout << "underline_position: " << face->underline_position * char_size / face->units_per_EM << std::endl;
  std::cout << "descender:    " << face->descender * char_size / face->units_per_EM << std::endl;
  std::cout << "height:       " << face->height * char_size / face->units_per_EM << std::endl;
  std::cout << std::endl;
  std::cout << "width:        " << (F26Dot6(glyph->metrics.width)) << std::endl;
  std::cout << "height:       " << (F26Dot6(glyph->metrics.height)) << std::endl;
  std::cout << std::endl;
  std::cout << "horiBearingX: " << (F26Dot6(glyph->metrics.horiBearingX)) << std::endl;
  std::cout << "horiBearingY: " << (F26Dot6(glyph->metrics.horiBearingY)) << std::endl;
  std::cout << "horiAdvance:  " << (F26Dot6(glyph->metrics.horiAdvance)) << std::endl;
  std::cout << std::endl;
  std::cout << "vertBearingX: " << (F26Dot6(glyph->metrics.vertBearingX)) << std::endl;
  std::cout << "vertBearingY: " << (F26Dot6(glyph->metrics.vertBearingY)) << std::endl;
  std::cout << "vertAdvance:  " << (F26Dot6(glyph->metrics.vertAdvance)) << std::endl;

  Renderer renderer(std::move(face), 1024 * 3, 128);
  renderer.draw_string(U"ABC あいうえお 漢字 αβγ '　'<=全角sp "
                       U"か" U"\u3099"
                       U"ほ" U"\u309A"
                       U"a" "\u0308"
                       U"<=結合文字(がぽä)");
  export_as_bmp(renderer.width, renderer.height, &renderer.data[0]);

  //glyph->advance;
  //metrics.horiBearingX
  //glyph->bitmap.width;
}
