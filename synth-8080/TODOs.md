# TODOs:

1. [x] write minimal control server
2. [x] write output node
3. [x] test VCO, minimal Control server, and output Node
4. [x] write a default implementation of a sample generator that take a lambda to generate samples || a macro that writes the boiler plate
    - NOTE: uses a function that takes callbacks.
5. [ ] write other modules
    - [x] ADBDR
    - [x] ADSR
    - [ ] Audio In
    - [x] Chorus (probably needs some fixing but its hard to tell with just a pure wave form and no knobs)
    - [x] Delay (same as echo just a different implementation)
    - [x] Echo
    - [ ] Gain/Attenuator
    - [x] LFO
    - [ ] Mid-Pass
    - [x] Output
    - [ ] Reverb
    - [x] VCO
6. [x] write alternate oscilators
    - [x] sine
    - [x] square
    - [x] triangle
    - [x] saw-tooth
    - [x] overtones oscilator
7. [x] add tanh to all inputs that except mutiple signals.
8. [ ] finish controller
9. [ ] write the code for the micro-controller to read the controlles
10. [ ] add a "get state" http end point for the tauri app to query
11. [ ] write tauri GUI front end
12. [ ] write ansible play book to install & setup\configure all this on a ras-pi (including flashing the micro-controller)
13. [ ] write midi inputs
