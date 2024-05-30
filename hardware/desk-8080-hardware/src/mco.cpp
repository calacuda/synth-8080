#include <Arduino.h>
#include <ArduinoJson.h>
#include "mco.h"
#include "common.h"

// double mco_vol = 0.0;
double output_vol = 0.5;
char *wave_form = "Sin";

void mco_display() {
    display_header("VCO");
    display_left_knob("vol.", output_vol);

    // display wave_form
    u8g2.setCursor(screen_size_x * 0.5, screen_size_y * 0.5);
    u8g2.print((char*) wave_form);
}

void mco_set_values(JsonDocument json) {
    output_vol = json["vol"];
    const char *wf = json["wave-form"];

    strncpy(wave_form, wf, 3);

}

// void mco_single_press() {
//     // change wave form
//     send_msg("MCO", 0, "wave-form-next");
// }

// void mco_double_press() {
//     // NOTHING
// }

void mco_hold_press() {
    // toggle over tones
    send_msg("MCO", 0, "overtones-toggle");
}

// knobs
void mco_left_knob(RotaryEncoder::Direction encoder_direction) {
    // output volume
    if (encoder_direction == RotaryEncoder::Direction::CLOCKWISE) {
        send_msg("MCO", 0, "vol", true);
    } else if (encoder_direction == RotaryEncoder::Direction::COUNTERCLOCKWISE) {
        send_msg("MCO", 0, "vol", false);
    }
}

void mco_right_knob(RotaryEncoder::Direction encoder_direction) {
    // change wave form
    if (encoder_direction == RotaryEncoder::Direction::CLOCKWISE) {
        send_msg("MCO", 0, "wave-form-next");
    } else if (encoder_direction == RotaryEncoder::Direction::COUNTERCLOCKWISE) {
        send_msg("MCO", 0, "wave-form-prev");
    }
}

Tab mk_mco_tab() {
    return Tab {
        mco_display,
        mco_set_values,
        mco_left_knob,
        mco_right_knob,
        mco_hold_press,
        mco_hold_press,
        // "mco",
    };
}