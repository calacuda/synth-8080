#include <Arduino.h>
#include <Wire.h>
#include <U8g2lib.h>
#include <cmath>
// #include <REncoder.h>
#include <RotaryEncoder.h>
#include <I2S.h>
#include "OneButton.h"
#include "common.h"
#include "mco.h"
#include "lowpass.h"

#define MCO 0
#define LOW_PASS 1
#define CHORUS 2
#define OVER_DRIVE 3
#define REVERB 4
#define ECHO 5
#define LFO_1 6
#define LFO_2 7
#define LFO_3 8
#define LFO_4 9

#define N_TABS 10

#define LED_PIN  14
#define BUTTON_PIN  15
#define LEFT_CLK_PIN 8
#define LEFT_DT_PIN 9
#define RIGHT_CLK_PIN 10
#define RIGHT_DT_PIN 11
#define LEFT_BUTTON_PIN 12  
#define RIGHT_BUTTON_PIN 13 

// int screen_middle[2] = { screen_size_x / 2, screen_size_y / 2 };
bool connected;
OneButton button(BUTTON_PIN);
// REncoder left_encoder(LEFT_CLK_PIN, LEFT_DT_PIN);
// REncoder right_encoder(RIGHT_CLK_PIN, RIGHT_DT_PIN);
RotaryEncoder left_encoder(LEFT_CLK_PIN, LEFT_DT_PIN, RotaryEncoder::LatchMode::TWO03);
RotaryEncoder right_encoder(RIGHT_CLK_PIN, RIGHT_DT_PIN, RotaryEncoder::LatchMode::TWO03);

OneButton left_button(LEFT_BUTTON_PIN);
OneButton right_button(RIGHT_BUTTON_PIN);
// REncoder::Event left_knob_history[2];
// REncoder::Event right_knob_history[2];
int left_pos;
int right_pos;

struct Tabs {
    Tab tabs[N_TABS];
    int i;
};

Tabs tabs;

void request_data();

void wait_for_con() {
    u8g2.clearBuffer();

    while (!Serial) {
        analogWrite(LED_PIN, 0);
        delay(750);
        analogWrite(LED_PIN, 1);
        delay(750);
    }

    analogWrite(LED_PIN, 1);
    connected = true;

    request_data();
}

void display_tab() {
    u8g2.clearBuffer();
    tabs.tabs[tabs.i].display();

    u8g2.sendBuffer();
}

void knobs() {
    // REncoder::Event encoder_ev_left = left_encoder.reState();
    // REncoder::Event encoder_ev_right = right_encoder.reState();

    // if (left_encoder.getPosition() % 2) {
    //     tabs.tabs[tabs.i].left_knob(encoder_ev_left);
    // }
    // if (right_encoder.getPosition() % 2) {
    //     tabs.tabs[tabs.i].right_knob(encoder_ev_right);
    // }
    left_encoder.tick();
    right_encoder.tick();

    int new_left_pos = left_encoder.getPosition();
    int new_right_pos = right_encoder.getPosition();

    if (left_pos != new_left_pos) {
        // Serial.print("pos:");
        // Serial.print(newPos);
        // Serial.print(" dir:");
        // Serial.println((int)(encoder.getDirection()));
        tabs.tabs[tabs.i].left_knob(left_encoder.getDirection());
        left_pos = new_left_pos;
    }

    if (right_pos != new_right_pos) {
        // Serial.print("pos:");
        // Serial.print(newPos);
        // Serial.print(" dir:");
        // Serial.println((int)(encoder.getDirection()));
        tabs.tabs[tabs.i].right_knob(right_encoder.getDirection());
        right_pos = new_right_pos;
    }
}

void single_click() {
    // Connection editor/maker
}

void double_click() {
    // Connection editor/maker
}

void hold() {
    // Connection editor/maker
}

void read_serial_in() {
    if (Serial.available()) {
        String json = Serial.readStringUntil('/n');
        JsonDocument doc;

        DeserializationError error = deserializeJson(doc, json);

        if (error) {
            send_log("failed to deserialize the received JSON data");
            return;
        }

        tabs.tabs[tabs.i].set_values(doc);
    }
}

void request_data() {
    switch (tabs.i) {
        case MCO:
            send_msg("mco");
            break;
        
        case LOW_PASS:
            send_msg("filter");
            break;

        case CHORUS:
            send_msg("chorus");
            break;

        case OVER_DRIVE:
            send_msg("overdrive");
            break;

        case REVERB:
            send_msg("reverb");
            break;

        case ECHO:
            send_msg("echo");
            break;

        case LFO_1:
            send_msg("lfo", 0);
            break;
        
        case LFO_2:
            send_msg("lfo", 1);
            break;

        case LFO_3:
            send_msg("lfo", 2);
            break;

        case LFO_4:
            send_msg("lfo", 3);
            break;
        
        default:
            break;
    }
}

void left_click() {
    tabs.i = (tabs.i - 1) % N_TABS;

    request_data();
}

void right_click() {
    tabs.i = (tabs.i + 1) % N_TABS;

    request_data();
}

void setup() {
    // put your setup code here, to run once:
    Serial.begin(115200);
    connected = false;
    left_pos = 0;
    right_pos = 0;
    pinMode(LED_PIN, OUTPUT);
    digitalWrite(LED_PIN, LOW);

    Wire.begin();

    u8g2.begin();
    u8g2.setFont(u8g2_font_cu12_h_symbols);

    // Clear the buffer.
    u8g2.clearBuffer();

    // for (int i = 0; i < N_TABS; i++) {
    //     tabs.tabs[i] = i;
    // }
    tabs.tabs[MCO] = mk_mco_tab();
    tabs.tabs[LOW_PASS] = mk_low_pass_tab();

    tabs.i = 0;

    button.attachDoubleClick(double_click);
    button.attachClick(single_click);
    button.attachLongPressStop(hold);
    // brings up a menu that forges a new connection.
    // button.attachMultiClick(connection_maker); // TODO

    left_button.attachClick(left_click);
    // left_encoder.setMinEncoderPosition(-2);
    // left_encoder.setMaxEncoderPosition(3);

    right_button.attachClick(right_click);
    // right_encoder.setMinEncoderPosition(-2);
    // right_encoder.setMaxEncoderPosition(3);

    // TODO: add second thread for audio.
}

void loop() {
    // put your main code here, to run repeatedly:
    if (!connected) {
        // wait for connection
        wait_for_con();
    }

    // display tab information
    display_tab();

    // read button input & do stuff
    knobs();

    button.tick();
    left_button.tick();
    right_button.tick();

    // read serial input & set values
    read_serial_in();

    delay(1);
}