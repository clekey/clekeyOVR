//
// Created by anatawa12 on 2022/09/04.
//

#ifndef CLEKEY_OVR_FREETYPERENDERER_H
#define CLEKEY_OVR_FREETYPERENDERER_H

#include <GL/glew.h>
#include "oglwrap/oglwrap.h"
#include "../Freetype.h"
#include <vector>
#include <unordered_map>

constexpr char32_t UNDEFINED_CHAR = 0xFFFFFFFF;

struct TextureMetrics {
  // in-texture cursor location
  uint16_t cursorX;
  uint16_t cursorY;
  // candidate for next cursorY
  uint16_t nextCursorY;
  // texture width = height
  uint16_t texSize;
};

struct FreetypeRendererTexture;
class GlyphInfo;

class FreetypeRenderer {
  freetype::Freetype ft;
  std::vector<freetype::Face> fonts;
  std::vector<FreetypeRendererTexture> textures;
  std::unordered_map<char32_t, GlyphInfo> glyphs;

  TextureMetrics metrics;

  gl::Program shaderProgram;
  gl::VertexAttrib vertexPosAttrib;
  gl::VertexAttrib vertexUVAttrib;
  gl::VertexAttrib vertexColorAttrib;
  gl::UniformSampler uniformFontTexture;
  gl::VertexArray vertexArray;
  gl::ArrayBuffer vertexBuffer;
  gl::IndexBuffer indexBuffer;

  FreetypeRenderer(gl::Program shaderProgram, gl::VertexAttrib vertexPosAttrib,
                   gl::VertexAttrib vertexUvAttrib, gl::VertexAttrib vertexColorAttrib,
                   gl::UniformSampler uniformFontTexture, gl::VertexArray vertexArray,
                   gl::ArrayBuffer vertexBuffer, gl::IndexBuffer indexBuffer);

  const GlyphInfo &tryLoadGlyphOf(char32_t c, bool *successful);

public:
  static FreetypeRenderer create();
  void addFontType(const char* path);

  /// @returns true if found
  bool loadGlyphOf(char32_t c);

  void addString(std::u8string string, glm::vec2 pos, glm::vec3 color, float size);
  void doDraw();

  FreetypeRenderer(FreetypeRenderer &&) noexcept;

  ~FreetypeRenderer();
};

#endif //CLEKEY_OVR_FREETYPERENDERER_H
