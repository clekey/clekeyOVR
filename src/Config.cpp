//
// Created by anatawa12 on 2022/09/18.
//

#include "Config.h"
#include "global.h"
#include <fstream>
#include <iostream>

#include "nlohmann/json.hpp"

using json = nlohmann::basic_json<>;

template<typename T>
void tryGetTo(T &variable, json j, const typename json::object_t::key_type &key) {
  try {
    j.at(key).get_to(variable);
  } catch (json::type_error &) {
  } catch (json::out_of_range &) {
  }
}

#define TRY_GET_TO(p, j, key) tryGetTo((p).key, j, #key)

void to_json(json &j, const OverlayPositionConfig &p) {
  j = json{
      {"pitch",      p.pitch},
      {"yaw",        p.yaw},
      {"distance",   p.distance},
      {"widthRadio", p.widthRadio},
      {"alpha",      p.alpha},
  };
}

void from_json(const json &j, OverlayPositionConfig &p) {
  TRY_GET_TO(p, j, pitch);
  TRY_GET_TO(p, j, yaw);
  TRY_GET_TO(p, j, distance);
  TRY_GET_TO(p, j, widthRadio);
  TRY_GET_TO(p, j, alpha);
}

void to_json(json &j, const CleKeyConfig &p) {
  j = json{
      {"leftRing",   p.leftRing},
      {"rightRing",  p.rightRing},
      {"completion", p.completion},
  };
}

OverlayPositionConfig::OverlayPositionConfig(float yaw, float pitch, float distance, float widthRadio, float alpha)
    : yaw(yaw), pitch(pitch), distance(distance), widthRadio(widthRadio), alpha(alpha) {}

CleKeyConfig::CleKeyConfig():
  leftRing{+6.0885f, -18.3379f, .75f, .2f, 1.0f},
  rightRing{-6.0885f, -18.3379f, .75f, .2f, 1.0f},
  completion{0.0f, -26.565f, .75f, .333f, 1.0f}
{}

void from_json(const json &j, CleKeyConfig &p) {
  TRY_GET_TO(p, j, leftRing);
  TRY_GET_TO(p, j, rightRing);
  TRY_GET_TO(p, j, completion);
}

namespace {
std::filesystem::path getConfigPath() {
  return getConfigDir() / "config.json";
}

void doLoadConfig(CleKeyConfig &config) {
  auto configPath = getConfigPath();
  std::fstream configStream(configPath, std::ios::in | std::ios::out | std::ios::binary);
  try {
    json::parse(configStream).get_to(config);
  } catch (std::exception &ex) {
    std::cout << "reading config: " << ex.what() << std::endl;
    config = {};
  }
}
}

CleKeyConfig loadConfig(CleKeyConfig &config) {
  doLoadConfig(config);
  try {
    auto configPath = getConfigPath();
    json configJson = config;
    std::ofstream configStream(configPath, std::ios::out | std::ios::trunc);
    configStream << std::setw(2) << configJson;
    configStream.close();
  } catch (std::exception &ex) {
    std::cout << "writing config: " << ex.what() << std::endl;
  }
  return config;
}
