//
// Created by anatawa12 on 2022/09/03.
//

#include "CursorCircleRenderer.h"
#include "glutil.h"

std::unique_ptr<CursorCircleRenderer> CursorCircleRenderer::create() {
  return std::make_unique<CursorCircleRenderer>();
}

void CursorCircleRenderer::draw(
    SkCanvas *canvas,
    SkPoint center,
    float radius,
    glm::vec2 stick,
    glm::vec4 color
) {
  SkPaint backRing;
  backRing.setAntiAlias(true);
  backRing.setColor(Color4fFromVec4(color));
  backRing.setStyle(SkPaint::kFill_Style);
  canvas->drawCircle(center + SkPoint{stick.x, -stick.y} * (radius / 3), radius * 0.25f, backRing);
}
