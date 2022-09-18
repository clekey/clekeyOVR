//
// Created by anatawa12 on 2022/09/18.
//

#ifndef CLEKEY_OVR_CONFIG_H
#define CLEKEY_OVR_CONFIG_H

#define DELETE_DEFAULT_BUT_KEEP_COPY(Class) \
  Class() = delete; \
  Class(Class &) = default; \
  Class &operator=(const Class &) = default;

#include "glm/glm.hpp"

struct OverlayPositionConfig {
  // in degree
  float yaw;
  float pitch;
  // in meter
  float distance;
  // width = distance * witchRadio
  float widthRadio;
  float alpha;

  DELETE_DEFAULT_BUT_KEEP_COPY(OverlayPositionConfig)
  OverlayPositionConfig(float yaw, float pitch, float distance, float widthRadio, float alpha);
};

struct RingOverlayConfig {
  OverlayPositionConfig position;
  glm::vec3 centerColor = glm::vec4(0.83, 0.83, 0.83, 1.0);
  glm::vec3 backgroundColor = glm::vec4(0.686, 0.686, 0.686, 1.0);
  glm::vec3 edgeColor = glm::vec4(1.0, 1.0, 1.0, 1.0);
  glm::vec3 normalCharColor = {0.0, 0.0, 0.0};
  glm::vec3 unSelectingCharColor = {0.5, 0.5, 0.5};
  glm::vec3 selectingCharColor = {0.0, 0.0, 0.0};
};

struct CompletionOverlayConfig {
  OverlayPositionConfig position;
  glm::vec3 backgroundColor;
  glm::vec3 inputtingCharColor;
};

struct CleKeyConfig {
  RingOverlayConfig leftRing;
  RingOverlayConfig rightRing;
  CompletionOverlayConfig completion;
public:
  CleKeyConfig();
};

CleKeyConfig loadConfig(CleKeyConfig &config);

#endif //CLEKEY_OVR_CONFIG_H
