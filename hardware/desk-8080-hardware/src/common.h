#ifndef COMMON_H
#define COMMON_H

#include <Arduino.h>
#include <U8g2lib.h>
// #include <REncoder.h>
#include <RotaryEncoder.h>
#include <ArduinoJson.h>

typedef void (*DisplayFunc)();
typedef void (*SetValuesFunc)(JsonDocument json);
typedef void (*LeftKnobFunc)(RotaryEncoder::Direction encoder_direction);
typedef void (*RightKnobFunc)(RotaryEncoder::Direction encoder_direction);
typedef void (*LeftKnobHoldFunc)();
typedef void (*RightKnobHoldFunc)();

struct Tab {
    DisplayFunc display;
    SetValuesFunc set_values;
    LeftKnobFunc left_knob;
    RightKnobFunc right_knob;
    LeftKnobHoldFunc left_knob_hold;
    RightKnobHoldFunc right_knob_hold;
    // char *tab_name;
};

const int screen_size_x = 128;
const int screen_size_y = 64;

void display_left_knob(char *knob_name, double value);
void display_right_knob(char *knob_name, double value);
void display_header(char *header);
// void two_knobs(String header, String left_knob_name, double left_knob_val, String right_knob_name, double right_knob_val);
void send_msg(char *mod, uint8_t index, char *control, bool increase);
void send_msg(char *mod, uint8_t index, char *control);
void send_msg(char *mod, uint8_t index);
void send_msg(char *mod);
void send_log(char *msg);
void nothing();

extern U8G2_SH1106_128X64_NONAME_F_HW_I2C u8g2;

#endif