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
#define I2S_BCLK_PIN 21
#define I2S_DATA_PIN 20
#define I2S_MCLK_PIN 19

#define ATTACK_PIN 28
#define DECAY_PIN 27
#define SUSTAIN_PIN 26

// int screen_middle[2] = { screen_size_x / 2, screen_size_y / 2 };
bool connected;
OneButton button(BUTTON_PIN);
// REncoder left_encoder(LEFT_CLK_PIN, LEFT_DT_PIN);
// REncoder right_encoder(RIGHT_CLK_PIN, RIGHT_DT_PIN);
// RotaryEncoder left_encoder(LEFT_CLK_PIN, LEFT_DT_PIN, RotaryEncoder::LatchMode::TWO03);
// RotaryEncoder right_encoder(RIGHT_CLK_PIN, RIGHT_DT_PIN, RotaryEncoder::LatchMode::TWO03);
// A pointer to the dynamic created rotary encoder instance.
// This will be done in setup()
RotaryEncoder *left_encoder = nullptr;
RotaryEncoder *right_encoder = nullptr;

OneButton left_button(LEFT_BUTTON_PIN);
OneButton right_button(RIGHT_BUTTON_PIN);
// REncoder::Event left_knob_history[2];
// REncoder::Event right_knob_history[2];
double attack = 0.0;
double decay = 0.0;
double sustain = 0.0;

// long sample = 0;
int sample = 0;
uint8_t sample_bytes[10];
bool play;

I2S audio(OUTPUT);

struct Tabs {
    Tab tabs[N_TABS];
    int i;
};

Tabs tabs;

void request_data();

void wait_for_con() {
    u8g2.clearBuffer();
    u8g2.sendBuffer();

    while (!Serial1) {
        analogWrite(LED_PIN, 0);
        delay(750);
        analogWrite(LED_PIN, 1);
        delay(750);
    }

    analogWrite(LED_PIN, 1);
    // analogWrite(LED_PIN, 0);
    connected = true;

    request_data();
}

void display_tab() {
    u8g2.clearBuffer();
    tabs.tabs[tabs.i].display();

    u8g2.sendBuffer();
}

void knobs() {
    tabs.tabs[tabs.i].left_knob((int) left_encoder->getDirection());

    tabs.tabs[tabs.i].right_knob((int) right_encoder->getDirection());

    double attack_mesurement = 1.0 - analogRead(ATTACK_PIN) / 1023.0;
    double decay_mesurement = 1.0 - analogRead(DECAY_PIN) / 1023.0;
    double sustain_mesurement = 1.0 - analogRead(SUSTAIN_PIN) / 1023.0;
    // Serial.println(attack_mesurement);

    if (abs(attack_mesurement - attack) > 1.0) {
        send_msg("filter", 0, "set-attack", attack_mesurement);
    }

    if (abs(decay_mesurement - decay) > 1.0) {
        send_msg("filter", 0, "set-decay", decay_mesurement);
    }

    if (abs(sustain_mesurement - sustain) > 1.0) {
        send_msg("filter", 0, "set-sustain", sustain_mesurement);
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
    if (Serial1.available()) {
        String json = Serial.readStringUntil('/n');
        JsonDocument doc;

        DeserializationError error = deserializeJson(doc, json);

        if (error) {
            send_log("[ERROR] failed to deserialize the received JSON data");
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

void left_knob_check() {
    left_encoder->tick(); // just call tick() to check the state.
}

void right_knob_check() {
    right_encoder->tick(); // just call tick() to check the state.
}

void setup1() {
    // put your setup code here, to run once:
    Serial1.begin(115200);
    connected = false;

    pinMode(LED_PIN, OUTPUT);
    digitalWrite(LED_PIN, LOW);

    // PinMode rotary_mode = INPUT_PULLUP;
    
    // pinMode(LEFT_CLK_PIN, rotary_mode);
    // pinMode(LEFT_DT_PIN, rotary_mode);
    pinMode(ATTACK_PIN, INPUT);
    pinMode(DECAY_PIN, INPUT);
    pinMode(SUSTAIN_PIN, INPUT);

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

    left_encoder = new RotaryEncoder(LEFT_DT_PIN, LEFT_CLK_PIN, RotaryEncoder::LatchMode::FOUR0);

    // register interrupt routine
    attachInterrupt(digitalPinToInterrupt(LEFT_CLK_PIN), left_knob_check, CHANGE);
    attachInterrupt(digitalPinToInterrupt(LEFT_DT_PIN), left_knob_check, CHANGE);

    right_encoder = new RotaryEncoder(RIGHT_DT_PIN, RIGHT_CLK_PIN, RotaryEncoder::LatchMode::FOUR0);

    // register interrupt routine
    attachInterrupt(digitalPinToInterrupt(RIGHT_CLK_PIN), right_knob_check, CHANGE);
    attachInterrupt(digitalPinToInterrupt(RIGHT_DT_PIN), right_knob_check, CHANGE);
    // play = false;
}

void loop1() {
    // put your main code here, to run repeatedly:
    if (!Serial1) {
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

    // if (Serial.available() >= 4) {
    //     uint8_t buf[10];
    //     Serial.readBytes(buf, 4);

    //     sample = (((long) buf[0]) << 24) + (((long) buf[1]) << 16) + (((long) buf[2]) << 8) + ((long) buf[3]);

    //     // play = true;
    //     // rp2040.fifo.push(sample);
    //     audio.write32(sample, sample);
    // }

    // delay(1);
}

// int counter = 0;

void setup() {
    Serial.begin(921600); // 230400, 921600
    // Serial.begin(115200);
    // Serial.setTimeout(1000);

    // set up i2s
    audio.setBCLK(I2S_BCLK_PIN);
    audio.setDATA(I2S_DATA_PIN);
    audio.setMCLK(I2S_MCLK_PIN);
    audio.setBitsPerSample(32);
    audio.setFrequency(24000.0);
    // audio.setMCLKmult(64);
    // audio.setSysClk(24000);
    // audio.swapClocks();
    // audio.setLSBJFormat();
    // Serial.print('\n');                           

    // while (Serial.available() < 4) {}
    // uint8_t buf[10];
    Serial.readBytes(sample_bytes, 4);

    // sample = (((int) buf[0]) << 8) + ((int) buf[1]);
    sample = (((long) sample_bytes[0]) << 24) + (((long) sample_bytes[1]) << 16) + (((long) sample_bytes[2]) << 8) + ((long) sample_bytes[3]);

    audio.begin();
    // digitalWrite(LED_BUILTIN, LOW);
    // new_sample();
}

void loop() {
    // if (Serial.available() % 4 == 0 && Serial.available()) {
    // while (Serial.available() >= 4) {
        // I2S out
        // new_sample();
        // uint8_t buf[10];
        // sample = 0;

        // Serial.readBytes(buf, 4);

        // sample = (((long) buf[0]) << 24) + (((long) buf[1]) << 16) + (((long) buf[2]) << 8) + ((long) buf[3]);
    // if (Serial.available() >= 4) {
        // uint8_t buf[10];
        // Serial.readBytes(buf, 4);

        // sample = (((long) buf[0]) << 24) + (((long) buf[1]) << 16) + (((long) buf[2]) << 8) + ((long) buf[3]);
    // if (Serial.available() >= 4) {

    //     play = true;
    // }

    // if (play) {
        // play = false;
        // audio.write32(sample, sample);
    // }
    // if (rp2040.fifo.available()) {
        // long samp = long(rp2040.fifo.pop());

    // uint8_t buf[10];
    // Serial.readBytes(buf, 4);
    // sample = (((long) buf[0]) << 24) + (((long) buf[1]) << 16) + (((long) buf[2]) << 8) + ((long) buf[3]);
    
    while (true) {   
        // digitalWrite(LED_PIN, HIGH);

        // audio.write(sample_bytes, 4);
        // audio.write(sample_bytes, 4);
        // // audio.flush();
        // audio.write(sample, false);                               
        // audio.write(sample, false);
        audio.write32(sample, sample);
        // TODO: send synth signal
        // Serial.write('\n');                           
        // audio.write32(0, 0);
        
        while (Serial.available() < 4) {}
        // audio.flush();

        // uint8_t buf[10];
        Serial.readBytes(sample_bytes, 4);

        // sample = (((int) buf[0]) << 8) + ((int) buf[1]);
        long sample = (((long) sample_bytes[0]) << 24) + (((long) sample_bytes[1]) << 16) + (((long) sample_bytes[2]) << 8) + ((long) sample_bytes[3]);
        // audio.flush();
        // audio.write(sample, true);
        // sample = new_sample;
    }
        // analogWrite(LED_PIN, 1);
    // }

        // audio.flush();
    // } 
        // audio.write(sample, true);
        // delay(500);
        // digitalWrite(LED_PIN, HIGH);
        // delay(500);
        // if (counter == 24000) {
        //     digitalWrite(LED_PIN, HIGH);
        // }
        
        // counter ++;
    // }
    // } else {
        //  digitalWrite(LED_PIN, HIGH);
        // digitalWrite(LED_BUILTIN, LOW);
    // }
}
