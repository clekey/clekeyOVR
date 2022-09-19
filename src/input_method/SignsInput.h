//
// Created by anatawa12 on 2022/09/14.
//

#ifndef CLEKEY_OVR_SIGNSINPUT_H
#define CLEKEY_OVR_SIGNSINPUT_H

#include "IInputMethod.h"

class SignsInput : public AbstractInputMethod {
public:
  SignsInput();

  InputNextAction onInput(glm::i8vec2) override;

  InputNextAction onHardInput(HardKeyButton) override;
};


#endif //CLEKEY_OVR_SIGNSINPUT_H
