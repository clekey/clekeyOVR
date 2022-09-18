//
// Created by anatawa12 on 2022/09/18.
//

#ifndef CLEKEY_OVR_CONFIG_H
#define CLEKEY_OVR_CONFIG_H

struct OverlayPositionConfig {
  // in degree
  float yaw;
  float pitch;
  // in meter
  float distance;
  // width = distance * witchRadio
  float widthRadio;
  float alpha;

  OverlayPositionConfig() = delete;

  OverlayPositionConfig(OverlayPositionConfig &) = default;

  OverlayPositionConfig &operator=(const OverlayPositionConfig &) = default;

  OverlayPositionConfig(float yaw, float pitch, float distance, float widthRadio, float alpha);
};

struct CleKeyConfig {
  OverlayPositionConfig leftRing;
  OverlayPositionConfig rightRing;
  OverlayPositionConfig completion;
public:
  CleKeyConfig();
};

CleKeyConfig loadConfig(CleKeyConfig &config);

#endif //CLEKEY_OVR_CONFIG_H
