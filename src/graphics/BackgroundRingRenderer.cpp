//
// Created by anatawa12 on 2022/09/03.
//

#include <include/core/SkCanvas.h>

#include "BackgroundRingRenderer.h"
#include "glutil.h"
#include <memory>

std::unique_ptr<BackgroundRingRenderer> BackgroundRingRenderer::create() {
  return std::make_unique<BackgroundRingRenderer>();
}

void BackgroundRingRenderer::draw(
    SkSurface& surface,
    glm::vec4 centerColor,
    glm::vec4 backgroundColor,
    glm::vec4 edgeColor
) {
  SkCanvas *canvas = surface.getCanvas();
  auto center = SkPoint {float(surface.width()) / 2, float(surface.height()) / 2};
  auto radius = float(surface.width()) / 2;

  auto edgeWidth = radius * 0.04f;
  auto backgroundRadius = radius - edgeWidth / 2;

  // background color
  {
    SkPaint backRing;
    backRing.setAntiAlias(true);
    backRing.setColor(Color4fFromVec4(backgroundColor));
    backRing.setStyle(SkPaint::kFill_Style);
    canvas->drawCircle(center, backgroundRadius, backRing);
  }

  // draw edge
  {
    SkPaint edge;
    edge.setAntiAlias(true);
    edge.setColor(Color4fFromVec4(edgeColor));
    edge.setStyle(SkPaint::kStroke_Style);
    edge.setStrokeWidth(edgeWidth);

    // first, outer ring
    canvas->drawCircle(center, backgroundRadius, edge);

    // then diagonal lines
    auto backup = canvas->getLocalToDevice();
    {
      canvas->rotate(22.5, center.x(), center.y());
      auto p1 = center - SkPoint{backgroundRadius, 0};
      auto p2 = center - SkPoint{-backgroundRadius, 0};
      for (int i = 0; i < 8; ++i) {
        canvas->drawLine(p1, p2, edge);
        canvas->rotate(45, center.x(), center.y());
      }
    }
    canvas->setMatrix(backup);
  }

  // finally, draw the center circle
  {
    SkPaint centerCircle;
    centerCircle.setAntiAlias(true);
    centerCircle.setColor(Color4fFromVec4(centerColor));
    centerCircle.setStyle(SkPaint::kFill_Style);

    canvas->drawCircle(center, radius / 2, centerCircle);
  }
}
