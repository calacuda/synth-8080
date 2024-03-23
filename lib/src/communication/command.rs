use crate::{notes::Note, Float, ModuleId, OscType};
use serde::{Deserialize, Serialize};

/// commands that can be sent to the synth via uart or over a unix-socket
#[derive(Deserialize, Serialize, PartialEq)]
pub enum SynthCmd {
    /// requests the entire, absolute state of the synth
    GetState,
    /// identifies the sender
    Identify(SynthId),
    /// plays a note
    Play(Note), // note is encoded as 'c#4' as a string, for example.
    /// stops playing a note
    Stop(Note),
    /// sets the speed of LFO which lfo is defined by the u8
    LfoSpeed((ModuleId, Float)),
    /// sets the depth/volume of LFO which lfo is defined by the u8
    LfoDepth((ModuleId, Float)),
    /// sets the type of the LFO,
    LfoType((ModuleId, OscType)),
    /// set VCO volume
    VcoVol(Float),
    /// set VCO oscilator type,
    VcoType(OscType),
    /// sets the attack of the adbdr envelope filter
    AdbdrAtk(Float),
    /// sets the decay_1 of the adbdr envelope filter
    AdbdrD1(Float),
    /// sets the decay_2 of the adbdr envelope filter
    AdbdrD2(Float),
    /// sets the break of the adbdr envelope filter (ie. when decay_1 becomes decay_2)
    AdbdrBreak(Float),
    /// sets the attack of the AD envelope filter
    AdAtk(Float),
    /// sets the decay of the AD envelope filter
    AdDecay(Float),
    /// sets the attack of the ADSR envelope filter
    AdsrAtk(Float),
    /// sets the Decay of the ADSR envelope filter
    AdsrDecay(Float),
    /// sets the sustain of the ADSR envelope filter
    AdsrSus(Float),
    /// sets the speed of the echo
    EchoSpeed(Float),
    /// sets the decay of the echos
    EchoDecay(Float),
    /// sets the gain of the overdrive
    OdGain(Float),
    /// sets chourus speed
    ChorusSpeed(Float),
    /// sets chorus volume
    ChorusDecay(Float),
    /// sets gain for reverb generation
    ReverbGain(Float),
    /// sets reverb decay,
    ReverbDecay(Float),
    /// connects two Modules, the first module is the src the second is the dest
    Connect(ModuleId, u8, ModuleId, u8),
    /// connects two Modules, the first module is the src the second is the dest
    Disconnect(ModuleId, u8, ModuleId, u8),
    /// bend pitch by amount
    PitchBend(Float),
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
pub enum SynthId {
    /// the synth
    Synth,
    /// the display (displayes connections on a monitor)
    DisplayClient,
    /// audio input board
    AudioInBoard,
    /// board incharge of knobs and switches
    ControlsBoard,
}
