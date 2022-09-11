//
// Created by anatawa12 on 2022/09/04.
//

#include "FreetypeRenderer.h"
#include <utility>
#include "glutil.h"
#include "../utf8.h"
#include <unordered_set>
#include FT_BITMAP_H

//currently horizontal rendering is only supported
class GlyphInfo {
public:
  // TODO: consider add horizontal mode for thin glyphs?
  // char32_t c; // target char is not required
  // in 1.0 = 1em, for computing positions
  float bearingX;
  float bearingY;
  float width;
  float height;
  float advance;
  // in UV space unit
  float minU;
  float minV;
  float maxU;
  float maxV;
  // texture index
  int texture;
  // font index
  int font;
};

struct FreetypeRendererVertex {
  float pos[2];
  float uv[2];
  float color[3];
};

static_assert(sizeof(FreetypeRendererVertex) == sizeof(float) * (2 + 2 + 3));
static_assert(alignof(FreetypeRendererVertex) == alignof(float));

struct FreetypeRendererQuad {
  // (min,min), (max,min), (max,max), (min,max)
  FreetypeRendererVertex vertex[4];

  FreetypeRendererQuad(float size, glm::vec3 color, const GlyphInfo &glyph, glm::vec2 origin);
};

static_assert(sizeof(FreetypeRendererQuad) == sizeof(FreetypeRendererVertex) * 4);
static_assert(alignof(FreetypeRendererQuad) == alignof(FreetypeRendererVertex));

inline FreetypeRendererQuad::FreetypeRendererQuad(float size, glm::vec3 color, const GlyphInfo &glyph, glm::vec2 origin) : vertex{} {
  float bearingXScaled = glyph.bearingX * size;
  float bearingYScaled = glyph.bearingY * size;
  float widthScaled = glyph.width * size;
  float heightScaled = glyph.height * size;

  float minX = origin.x + bearingXScaled;
  float minY = origin.y + bearingYScaled - heightScaled;
  float maxX = origin.x + bearingXScaled + widthScaled;
  float maxY = origin.y + bearingYScaled;

  FreetypeRendererVertex v0{
      .pos = {minX, minY},
      .uv = {glyph.minU, glyph.minV},
      .color = {color.r, color.g, color.b}
  };
  FreetypeRendererVertex v1{
      .pos = {maxX, minY},
      .uv = {glyph.maxU, glyph.minV},
      .color = {color.r, color.g, color.b}
  };
  FreetypeRendererVertex v2{
      .pos = {maxX, maxY},
      .uv = {glyph.maxU, glyph.maxV},
      .color = {color.r, color.g, color.b}
  };
  FreetypeRendererVertex v3{
      .pos = {minX, maxY},
      .uv = {glyph.minU, glyph.maxV},
      .color = {color.r, color.g, color.b}
  };

  vertex[0] = v0;
  vertex[1] = v1;
  vertex[2] = v2;
  vertex[3] = v3;
}

/// a buffer body for one font texture
struct FreetypeRendererTexture {
  // texture id
  gl::Texture2D texture2D;
  // vertex buffer
  std::vector<FreetypeRendererQuad> buffer;
  // index buffer
  std::vector<GLuint> indices;

  FreetypeRendererTexture();
  ~FreetypeRendererTexture();
  FreetypeRendererTexture(FreetypeRendererTexture&& other) = default;
  FreetypeRendererTexture& operator=(FreetypeRendererTexture&& other) = default;

  void addQuad(FreetypeRendererQuad quad);
};

FreetypeRendererTexture::FreetypeRendererTexture() {
  // reserve for 256 chars
  buffer.reserve(256);
  indices.reserve(256 * 3 * 2);
}

void FreetypeRendererTexture::addQuad(FreetypeRendererQuad quad) {
  auto added_idx = buffer.size();
  buffer.push_back(quad);
  indices.reserve(3 * 2);
  indices.push_back(added_idx * 4 + 0);
  indices.push_back(added_idx * 4 + 1);
  indices.push_back(added_idx * 4 + 2);
  indices.push_back(added_idx * 4 + 0);
  indices.push_back(added_idx * 4 + 2);
  indices.push_back(added_idx * 4 + 3);
}

FreetypeRendererTexture::~FreetypeRendererTexture() = default;

void FreetypeRenderer::addFontType(const char *path) {
  fonts.push_back(std::move(ft.new_face(path, 0)));
}

std::unique_ptr<FreetypeRenderer> FreetypeRenderer::create() {
  auto program = compile_shader_program(
      "#version 330 core\n"
      "in vec2 vPos;\n"
      "in vec2 vUv;\n"
      "in vec3 vColor;\n"
      "\n"
      "out vec2 fUV;\n"
      "flat out vec3 fColor;\n"
      "\n"
      "void main() {\n"
      "    gl_Position.xy = vPos;\n"
      "    fUV = vUv;\n"
      "    fColor = vColor;\n"
      "}\n",
      "#version 330 core\n"
      "in vec2 fUV;\n"
      "flat in vec3 fColor;\n"
      "\n"
      "out vec4 color;\n"
      "\n"
      "uniform sampler2D fuFontTexture;\n"
      "\n"
      "void main() {\n"
      "    color = vec4(fColor, texture(fuFontTexture, fUV).r);\n"
      "}\n"
  );

  gl::Bind(program);

  gl::VertexAttrib vertexPosAttrib(program, "vPos");
  gl::VertexAttrib vertexUVAttrib(program, "vUv");
  gl::VertexAttrib vertexColorAttrib(program, "vColor");

  gl::UniformSampler uniformFontTexture(program, "fuFontTexture");

  gl::VertexArray vertexArray;
  gl::ArrayBuffer vertexBuffer;
  gl::IndexBuffer indexBuffer;

  gl::Bind(vertexArray);
  gl::Bind(vertexBuffer);
  gl::Bind(indexBuffer);


  vertexPosAttrib.enable();
  vertexUVAttrib.enable();
  vertexColorAttrib.enable();

  vertexPosAttrib.pointer(2, gl::DataType::kFloat, false, sizeof(FreetypeRendererVertex),
                          (void *) offsetof(FreetypeRendererVertex, pos));
  vertexUVAttrib.pointer(2, gl::DataType::kFloat, false, sizeof(FreetypeRendererVertex),
                         (void *) offsetof(FreetypeRendererVertex, uv));
  vertexColorAttrib.pointer(3, gl::DataType::kFloat, false, sizeof(FreetypeRendererVertex),
                            (void *) offsetof(FreetypeRendererVertex, color));

  auto res = new FreetypeRenderer{
      std::move(program),
      std::move(vertexPosAttrib),
      std::move(vertexUVAttrib),
      std::move(vertexColorAttrib),
      std::move(uniformFontTexture),
      std::move(vertexArray),
      std::move(vertexBuffer),
      std::move(indexBuffer),
  };
  return std::unique_ptr<FreetypeRenderer>(res);
}

void initTexture(TextureMetrics &metrics, gl::Texture2D &texture) {
  gl::Bind(texture);
  texture.upload(gl::kR8, metrics.texSize, metrics.texSize, gl::kRed, gl::kUnsignedByte, nullptr);
  texture.minFilter(gl::kLinear);
}

FreetypeRenderer::FreetypeRenderer(
    gl::Program shaderProgram, gl::VertexAttrib vertexPosAttrib,
    gl::VertexAttrib vertexUvAttrib, gl::VertexAttrib vertexColorAttrib,
    gl::UniformSampler uniformFontTexture, gl::VertexArray vertexArray,
    gl::ArrayBuffer vertexBuffer, gl::IndexBuffer indexBuffer
) : textures(1),
    metrics{},
    shaderProgram(std::move(shaderProgram)),
    vertexPosAttrib(std::move(vertexPosAttrib)),
    vertexUVAttrib(std::move(vertexUvAttrib)),
    vertexColorAttrib(std::move(vertexColorAttrib)),
    uniformFontTexture(std::move(uniformFontTexture)),
    vertexArray(std::move(vertexArray)),
    vertexBuffer(std::move(vertexBuffer)),
    indexBuffer(std::move(indexBuffer)) {
  // GL_MAX_TEXTURE_SIZE
  GLint texSize;
  glGetIntegerv(GL_MAX_TEXTURE_SIZE, &texSize);
  metrics.texSize = (uint16_t) std::min(texSize, 64 * 64);

  initTexture(metrics, textures.back().texture2D);
}

const GlyphInfo &FreetypeRenderer::tryLoadGlyphOf(char32_t c, bool *successful) {
  // if glyph is already defined, use it
  {
    auto glyph_iter = glyphs.find(c);
    if (glyph_iter != glyphs.end()) return glyph_iter->second;
  }

  FT_Glyph_Metrics glyphMetrics;
  FT_Bitmap bitmap;
  int font_index;
  FT_Bitmap_Init(&bitmap);
  bitmap.pitch = -1; // to make result image down to top
  if (c == UNDEFINED_CHAR) {
    auto &font = *fonts.begin();
    font.setPixelSizes(64, 64);
    font.loadGlyph(0, FT_LOAD_RENDER);
    // convert to without alignment
    ft.bitmap_convert(font->glyph->bitmap, bitmap, 0);
    glyphMetrics = font->glyph->metrics;
    font_index = 0;
  } else {
    auto iter = std::begin(fonts);
    auto end = std::end(fonts);
    font_index = 0;
    for (; iter != end; iter++, font_index++) {
      auto &font = *iter;
      auto charIndex = font.getCharIndex(c);
      if (charIndex != 0) {
        font.setPixelSizes(64, 64);
        font.loadGlyph(charIndex, FT_LOAD_RENDER);
        ft.bitmap_convert(font->glyph->bitmap, bitmap, 0);
        glyphMetrics = font->glyph->metrics;
        break;
      }
    }
    if (iter == end) {
      if (successful) *successful = false;
      return tryLoadGlyphOf(UNDEFINED_CHAR, nullptr);
    }
  }
  assert(-bitmap.pitch == bitmap.width);
  assert(bitmap.pixel_mode == FT_PIXEL_MODE_GRAY);
  gl::PixelStore(gl::kUnpackAlignment, 1);
  auto maxX = metrics.cursorX + bitmap.width;
  if (maxX > metrics.texSize) {
    // x-axis overflow: continue in next row
    metrics.cursorX = 0;
    // make space between line
    metrics.cursorY = metrics.nextCursorY + 1;
    maxX = bitmap.width;
  }

  auto maxY = metrics.cursorY + bitmap.rows;
  if (maxY > metrics.texSize) {
    // y-axis overflow: create new texture
    auto &texture = textures.emplace_back();
    initTexture(metrics, texture.texture2D);
    metrics.cursorX = 0;
    metrics.cursorY = 0;
    maxX = bitmap.width;
    metrics.nextCursorY = maxY = bitmap.rows;
  }
  metrics.nextCursorY = std::max((uint16_t) maxY, metrics.nextCursorY);

  auto texture_idx = textures.size() - 1;
  auto &texture = textures.back();

  gl::Bind(texture.texture2D);
  texture.texture2D.subUpload(
      metrics.cursorX, metrics.cursorY,
      (int) bitmap.width, (int) bitmap.rows,
      gl::kRed, gl::kUnsignedByte,
      bitmap.buffer);

  const float f26dot6toFloat = 64;
  const float oneEmInPixel = 64;

  auto &glyphInfo = glyphs[c];
  glyphInfo = {
      //
      .bearingX = (float)glyphMetrics.horiBearingX / f26dot6toFloat / oneEmInPixel,
      .bearingY = (float)glyphMetrics.horiBearingY / f26dot6toFloat / oneEmInPixel,
      .width = (float)bitmap.width / oneEmInPixel,
      .height = (float)bitmap.rows / oneEmInPixel,
      .advance = (float)glyphMetrics.horiAdvance / f26dot6toFloat / oneEmInPixel,
      // in UV space unit
      .minU = (float)metrics.cursorX / (float)metrics.texSize,
      .minV = (float)metrics.cursorY / (float)metrics.texSize,
      .maxU = (float)maxX / (float)metrics.texSize,
      .maxV = (float)maxY / (float)metrics.texSize,
      // texture index
      .texture = (int) texture_idx,
      .font = font_index,
  };

  // make space between char
  metrics.cursorX = maxX + 1;

  if (successful) *successful = true;

  return glyphInfo;
}

bool FreetypeRenderer::loadGlyphOf(char32_t c) {
  bool success;
  tryLoadGlyphOf(c, &success);
  return success;
}

void FreetypeRenderer::addString(const std::u8string &string, glm::vec2 pos, glm::vec3 color, float size) {
  for (const auto c: make_u8u32range(string)) {
    auto &glyph = tryLoadGlyphOf(c, nullptr);
    auto &tex = textures[glyph.texture];
    tex.addQuad({size, color, glyph, pos});
    pos.x += glyph.advance * size;
  }
}

glm::vec2 FreetypeRenderer::calcStringSize(const std::u8string &string) {
  std::unordered_set<int> fontIndexList{};
  float width = 0;
  for (const auto c: make_u8u32range(string)) {
    auto &glyph = tryLoadGlyphOf(c, nullptr);
    width += glyph.advance;
    fontIndexList.insert(glyph.font);
  }
  float height = 0;
  for (const auto &index: fontIndexList) {
    const auto &font = fonts[index];
    height = std::max(height, float(font->descender) / float(font->units_per_EM) + 1);
  }
  return {width, height};
}

void FreetypeRenderer::addCenteredString(const std::u8string& string, glm::vec2 pos, glm::vec3 color, float size, CenteredMode mode) {
  auto wh = calcStringSize(string);
  if (mode & CenteredMode::Horizontal)
    pos.x -= wh.x * size / 2;
  if (mode & CenteredMode::Vertical)
    pos.y -= wh.y * size / 2;
  addString(string, pos, color, size);
}

void FreetypeRenderer::addCenteredStringWithMaxWidth(const std::u8string& string, glm::vec2 pos, glm::vec3 color, float size, float maxWidth, CenteredMode mode) {
  auto wh = calcStringSize(string);
  size = std::min(size, maxWidth/wh.x);
  if (mode & CenteredMode::Horizontal)
    pos.x -= wh.x * size / 2;
  if (mode & CenteredMode::Vertical)
    pos.y -= wh.y * size / 2;
  addString(string, pos, color, size);
}

void FreetypeRenderer::doDraw() {
  gl::Use(shaderProgram);
  gl::Bind(vertexArray);
  uniformFontTexture.set(0);

  for (auto &item: textures) {
    if (!item.buffer.empty()) {
      gl::BindToTexUnit(item.texture2D, 0);
      gl::Bind(indexBuffer);
      indexBuffer.data(item.indices);
      gl::Bind(vertexBuffer);
      vertexBuffer.data(item.buffer);

      gl::DrawElements(gl::kTriangles, (GLsizei) item.indices.size(), gl::kUnsignedInt);
      item.indices.clear();
      item.buffer.clear();
    }
  }
}

FreetypeRenderer::~FreetypeRenderer() = default;
FreetypeRenderer::FreetypeRenderer(FreetypeRenderer &&) noexcept = default;
