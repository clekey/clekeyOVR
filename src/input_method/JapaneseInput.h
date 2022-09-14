//
// Created by anatawa12 on 2022/09/14.
//

#ifndef CLEKEY_OVR_JAPANESEINPUT_H
#define CLEKEY_OVR_JAPANESEINPUT_H

#include "IInputMethod.h"

class JapaneseInput : public AbstractInputMethod {
public:
  JapaneseInput();
  InputNextAction onInput(glm::i8vec2) override;
};

#endif //CLEKEY_OVR_JAPANESEINPUT_H
