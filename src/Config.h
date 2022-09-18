//
// Created by anatawa12 on 2022/09/18.
//

#ifndef CLEKEY_OVR_CONFIG_H
#define CLEKEY_OVR_CONFIG_H

#define DELETE_DEFAULT_BUT_KEEP_COPY(Class) \
  Class() = delete; \
  Class(Class &) = default; \
  Class &operator=(const Class &) = default;


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
};

struct CompletionOverlayConfig {
  OverlayPositionConfig position;
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
