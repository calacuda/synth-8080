#include <Arduino.h>
#include <ArduinoJson.h>
#include "lowpass.h"
#include "common.h"

double cutoff = 0.5;
double resonance = 0.5;

void lowpass_display() {
    display_header("LowPass");
    display_left_knob("cutoff", cutoff);
    display_right_knob("res", resonance);
}

void lowpass_set_values(JsonDocument json) {
    cutoff = json["cutoff"];
    resonance = json["resonance"];
}

// void lowpass_single_press() {
//     // NOTHING
// }

// void lowpass_double_press() {
//     // NOTHING
// }

// void lowpass_hold_press() {
//     // NOTHING
// }

// knobs
void lowpass_left_knob(RotaryEncoder::Direction encoder_direction) {
    if (encoder_direction == RotaryEncoder::Direction::CLOCKWISE) {
        send_msg("lowpass", 0, "cutoff", true);
    } else if (encoder_direction == RotaryEncoder::Direction::COUNTERCLOCKWISE) {
        send_msg("lowpass", 0, "cutoff", false);
    }
}

void lowpass_right_knob(RotaryEncoder::Direction encoder_direction) {
    if (encoder_direction == RotaryEncoder::Direction::CLOCKWISE) {
        send_msg("lowpass", 0, "resonance", true);
    } else if (encoder_direction == RotaryEncoder::Direction::COUNTERCLOCKWISE) {
        send_msg("lowpass", 0, "resonance", false);
    }
}

Tab mk_low_pass_tab() {
    return Tab {
        lowpass_display,
        lowpass_set_values,
        lowpass_left_knob,
        lowpass_right_knob,
        nothing,
        nothing,
        // "filter",
    };
}