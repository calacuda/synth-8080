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
        - [ ] make delay just delay the signal. so it can be used to offset lfos.
    - [x] Echo
    - [ ] Attenuator
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
    - [x] overtones oscillator
7. [x] add tanh to all inputs that except multiple signals.
8. [x] finish controller
9. [ ] write the code for the micro-controller to read the controls
    - [x] design layout of controls
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
15. [x] write midi inputs
    - [x] ~~write async input controller~~ Not necessary
    - [x] test with raspberry-pi pico || arduino pro-micro (at least one of them should be able to send midi input)
16. [x] make a new struct that holds a configurable number of VCOs and envelope filters. to achieve polyphony with "one" struct.
17. [ ] add ability to edit connections that are already made.
18. [ ] add ability to temporarily disconnect connections that are already made.
19. [ ] add IPC (over usix socket or maybe websockets, to be more crossplatform) so other processes can change the synths parameters.
20. [ ] -> add tauri events to change front end on synth-param changes <- (do this next)
21. [ ] make the sliders set the value they control to 50% at load
22. [ ] add scriptability (using a custom lisp dialect)
    - [ ] design said dialect
    - [ ] implement the basics
    - [ ] build a std-lib
    - [ ] add a IPC control for refreshing/reloading or even changing the loaded script (so a text editor can notify the synth on file saves.)
    - [ ] 
