#include <Arduino.h>
#include <BLEMidi.h>

bool pressed = false;

void setup() {
    // put your setup code here, to run once:
    Serial.begin(115200);
    pinMode(23, INPUT_PULLDOWN);

    Serial.println("Initializing bluetooth");
    BLEMidiServer.begin("Basic MIDI device");
    Serial.println("Waiting for connections...");
    // BLEMidiServer.enableDebugging();  // Uncomment if you want to see some debugging output from the library
}

void loop() {
    // put your main code here, to run repeatedly:
    if(BLEMidiServer.isConnected()) {             // If we've got a connection, we send an A4 during one second, at full velocity (127)
        // BLEMidiServer.noteOn(0, 69, 127);
        // delay(1000);
        // BLEMidiServer.noteOff(0, 69, 127);        // Then we stop the note and make a delay of one second before returning to the beginning of the loop
        // delay(1000);
        if (!pressed && digitalRead(23) == HIGH) {
            pressed = true;
            Serial.println("playing");
            BLEMidiServer.noteOn(0, 60, 127);
        } else if (pressed && digitalRead(23) == LOW) {
            pressed = false;
            Serial.println("stopping");
            BLEMidiServer.noteOff(0, 60, 127);
        }
    }
}