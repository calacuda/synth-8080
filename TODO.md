# TODOs:

1. [x] write minimal control server
2. [x] write output node
3. [x] test VCO, minimal Control server, and output Node
4. [x] write a default implementation of a sample generator that take a lambda to generate samples || a macro that writes the boiler plate
    - ~~NOTE: use a function that takes callbacks.~~
5. [ ] write other modules
    - [x] ADBDR
    - [x] ADSR
    - [x] AD
    - [ ] Audio In 
    - [x] Chorus (probably needs some fixing but its hard to tell with just a pure wave form and no knobs)
    - [x] Delay (same as echo just a different implementation)
    - [x] Echo
    - [ ] Attenuator (optional)
    - [x] LFO
    <!-- - [ ] Mid-Pass -->
    - [x] Output
    - [x] Reverb
    - [x] VCO
    - [x] overdrive
6. [x] write alternate oscilators
    - [x] sine
    - [x] square
    - [x] triangle
    - [x] saw-tooth
    - [x] overtones oscilator
7. [x] add tanh to all inputs that except mutiple signals.
8. [x] finish controller
9. [ ] write the code for the micro-controller to read the controlles
    - [x] design layout of controlles
    - [x] make circuit diagram
    - [x] design UART communication API
    - [ ] write code
        - [x] make enums for commands and impl to && from slice of U8s
10. [ ] add audio input (see no. 5 - `write other modules`)
    - NOTE: probably with a secondary micro-controller
11. [ ] add a "get state" http end point for the tauri app to query
    - NOTE: use unix socket instead, will be more reasorce efficient
12. [ ] write tauri GUI front end
13. [ ] write ansible play book to install & setup\configure all this on a ras-pi (including flashing the micro-controller)
14. [ ] build the housing
15. [ ] write midi inputs
    - [ ] write async input controller
    - [ ] test with raspberry-pi pico || arduino pro-micro (at least one of them should be able to send midi input)
