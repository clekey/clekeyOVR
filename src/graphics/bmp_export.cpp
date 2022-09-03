//
// Created by anatawa12 on 8/11/22.
//

#include "glutil.h"
#include <fstream>
#include <sstream>
#include <filesystem>
#include "bmp_export.h"

void export_as_bmp(gl::Texture2D &texture, GLint level) {
  const size_t header_size = 14 + 40;

  static int index = 0;

  gl::Bind(texture);
  GLint w = texture.width(level);
  GLint h = texture.height(level);

  std::vector<uint8_t> bmp_data(w * h * 4 + header_size);

  glGetTexImage(GL_TEXTURE_2D, level,
                GL_RGBA, GL_UNSIGNED_BYTE,
                &bmp_data[header_size]);
  check_gl_err(__func__);

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
  bmp_data[10] = header_size;
  bmp_data[11] = 0;
  bmp_data[12] = 0;
  bmp_data[13] = 0;

  // OS/2 bitmap header
  bmp_data[14] = 40;
  bmp_data[15] = 0;
  bmp_data[16] = 0;
  bmp_data[17] = 0;
  // width
  bmp_data[18] = (w >> 0) & 0xFF;
  bmp_data[19] = (w >> 8) & 0xFF;
  bmp_data[20] = (w >> 16) & 0xFF;
  bmp_data[21] = (w >> 24) & 0xFF;
  // height
  bmp_data[22] = (h >> 0) & 0xFF;
  bmp_data[23] = (h >> 8) & 0xFF;
  bmp_data[24] = (h >> 16) & 0xFF;
  bmp_data[25] = (h >> 24) & 0xFF;
  // planes = 1
  bmp_data[26] = 1;
  bmp_data[27] = 0;
  // bit per pixcel = 32 = 8 * 4
  bmp_data[28] = 32;
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

  for (int i = 0; i < w * h; ++i) {
    uint8_t r = bmp_data[header_size + i * 4 + 0];
    //uint8_t g = bmp_data[26 + i * 4 + 1];
    uint8_t b = bmp_data[header_size + i * 4 + 2];
    bmp_data[header_size + i * 4 + 0] = b;
    //bmp_data[26 + i * 4 + 1] = g;
    bmp_data[header_size + i * 4 + 2] = r;
  }

  std::filesystem::create_directories("frames");

  std::stringstream bmp_path_builder;
  bmp_path_builder << "frames/frame_" << std::setfill('0') << std::setw(5) << (index++) << ".bmp";
  std::string bmp_path = bmp_path_builder.str();
  std::ofstream bmp_file;
  bmp_file.open(bmp_path, std::ios::out | std::ios::binary);

  bmp_file.write(reinterpret_cast<const char *>(&bmp_data[0]), (size_t) bmp_data.size());

  bmp_file.close();
}
