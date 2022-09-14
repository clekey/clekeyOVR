//
// Created by anatawa12 on 2022/09/14.
//

#ifndef CLEKEY_OVR_ENGLISHINPUT_H
#define CLEKEY_OVR_ENGLISHINPUT_H

#include "IInputMethod.h"

class EnglishInput : public AbstractInputMethod {
public:
  EnglishInput();
  InputNextAction onInput(glm::i8vec2) override;
};


#endif //CLEKEY_OVR_ENGLISHINPUT_H
