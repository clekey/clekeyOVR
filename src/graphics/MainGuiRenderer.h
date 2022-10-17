//
// Created by anatawa12 on 8/11/22.
//

#ifndef CLEKEY_OVR_MAINGUIRENDERER_H
#define CLEKEY_OVR_MAINGUIRENDERER_H

#include <include/core/SkRefCnt.h>
#include <include/core/SkSurface.h>
#include <include/core/SkTypeface.h>

#include "BackgroundRingRenderer.h"
#include "CursorCircleRenderer.h"
#include "../OVRController.h"
//#include "FreetypeRenderer.h"
#include "../AppStatus.h"
#include <include/FontCollection.h>

class MainGuiRenderer {
public:
  static std::unique_ptr<MainGuiRenderer> create(glm::ivec2 size);

  void drawRing(
      const KeyboardStatus &status,
      LeftRight side,
      bool alwaysShowInCircle,
      const RingOverlayConfig &config,
      SkSurface& surface
  );

  void drawCenter(const KeyboardStatus &status, const CompletionOverlayConfig &config, SkSurface& surface);

  glm::ivec2 size;

  std::unique_ptr<BackgroundRingRenderer> backgroundRingRenderer;
  sk_sp<skia::textlayout::FontCollection> fonts;
  std::unique_ptr<CursorCircleRenderer> cursorCircleRenderer;
  //std::unique_ptr<FreetypeRenderer> ftRenderer;
};

#endif //CLEKEY_OVR_MAINGUIRENDERER_H
