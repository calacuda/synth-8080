#include <Arduino.h>
#include <cmath>
#include <U8g2lib.h>
#include <ArduinoJson.h>
#include "common.h"

U8G2_SH1106_128X64_NONAME_F_HW_I2C u8g2(U8G2_R0);
const double pi = 3.14159265;
int screen_middle[2] = { screen_size_x / 2, screen_size_y / 2 };

void display_left_knob(char *knob_name, double value) {
    // *might* need to multiply by negative 1
    double theta = -1.0 * ((value * 360.0) + 270.0) * (pi / 180.0);
    double scaler = (double) screen_size_y * 0.25 - 1.0;
    double unit_circle_point[2] = { cos( theta ) * scaler, sin( theta ) * scaler };
    int center[2] = { screen_middle[0] * 0.5, screen_middle[1] };

    // draw circle
    u8g2.drawCircle( center[0], center[1], scaler);

    // draw indicator line
    u8g2.drawLine(
        center[0],
        center[1],
        center[0] - round(unit_circle_point[0]),
        center[1] + round(unit_circle_point[1])
        );

    // display knob name
    u8g2.setCursor(0, screen_size_y);
    u8g2.print(knob_name);
}

void display_right_knob(char *knob_name, double value) {
    // *might* need to multiply by negative 1
    // Serial.println(value * 360.0);
    double theta = -1.0 * ((value * 360.0) + 270) * (pi / 180.0);
    double scaler = (double) screen_size_y * 0.25 - 1.0;
    double unit_circle_point[2] = { cos( theta ) * scaler, sin( theta ) * scaler };
    int center[2] = { (int) round(screen_size_x * 0.75), screen_middle[1] };

    // draw circle
    u8g2.drawCircle( center[0], center[1], scaler);

    // draw indicator line
    u8g2.drawLine(
        center[0],
        center[1],
        center[0] - round(unit_circle_point[0]),
        center[1] + round(unit_circle_point[1])
        );

    // display knob name
    u8g2.setCursor(screen_size_x * 0.5, screen_size_y);
    u8g2.print(knob_name);
}

void display_header(char *header) {
    // diplays module name on top.
    // u8g2.setCursor(0, 12);
    u8g2.drawStr(0, 12, header);
};

// void two_knobs(String header, String left_knob_name, double left_knob_val, String right_knob_name, double right_knob_val);

void send_msg(char mod[12] , uint8_t index, char control[50], bool increase) {
    // print json to Serial
    JsonDocument doc;

    doc["mod"] = mod;
    doc["index"] = index;
    doc["cmd"] = control;
    doc["increase"] = increase;

    serializeJson(doc, Serial1);
    Serial1.println();
}

void send_msg(char mod[12] , uint8_t index, char control[50], double ammount) {
    // print json to Serial
    JsonDocument doc;

    doc["mod"] = mod;
    doc["index"] = index;
    doc["cmd"] = control;
    doc["value"] = ammount;

    serializeJson(doc, Serial1);
    Serial1.println();
}

void send_msg(char mod[12] , uint8_t index, char control[50]) {
    // print json to Serial (but without a direction, used for controls that dont have directionality)
    JsonDocument doc;

    doc["mod"] = mod;
    doc["index"] = index;
    doc["cmd"] = control;

    serializeJson(doc, Serial1);
    Serial1.println();
}

void send_msg(char mod[12] , uint8_t index) {
    // print json to Serial (used to request data from synth)
    // JsonDocument doc;

    // doc["mod"] = mod;
    send_msg(mod, index, "get_data");
}

void send_msg(char mod[12] ) {
    // print json to Serial (used to request data from synth)
    // JsonDocument doc;

    // doc["mod"] = mod;
    send_msg(mod, 0);
}

void send_log(char log[2000]) {
    // print json to Serial (used to log data to the synths log)
    JsonDocument doc;

    doc["log-message"] = log;

    serializeJson(doc, Serial1);
    Serial1.println();

    // Serial.println("JSON?");
}

void nothing() {
    // NOTHING
}
