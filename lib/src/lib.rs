use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum_macros::{EnumIter, EnumString};

pub mod communication;
pub mod notes;

pub type ModuleId = u8;

#[cfg(not(feature = "double_float"))]
pub type Float = f32;
// pub const SAMPLE_RATE: u32 = 48_000;
// pub const SAMPLE_RATE: u32 = 44_100;
// pub const SAMPLE_RATE: u32 = 24_000;
// pub const SAMPLE_RATE: u32 = 22_050;
// pub const SAMPLE_RATE: u32 = 16_000;
#[cfg(not(feature = "wav_sample_rate"))]
pub const SAMPLE_RATE: u32 = 44_100;
#[cfg(feature = "double_float")]
pub type Float = f32;
// pub type Float = f64;
#[cfg(feature = "wav_sample_rate")]
pub const SAMPLE_RATE: u32 = 48_000;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, EnumIter, Hash, EnumString)]
pub enum OscType {
    #[serde(alias = "sine", alias = "sin")]
    Sine,
    #[serde(alias = "square", alias = "squ")]
    Square,
    #[serde(alias = "triangle", alias = "tri")]
    Triangle,
    #[serde(alias = "saw-tooth", alias = "sawtooth", alias = "saw")]
    SawTooth,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Connection {
    pub src_module: u8,
    pub src_output: u8,
    pub dest_module: u8,
    pub dest_input: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, EnumIter)]
pub enum ModuleType {
    Vco,
    Output,
    Lfo,
    Echo,
    #[serde(alias = "env")]
    EnvFilter,
    Chorus,
    Delay,
    #[serde(alias = "od")]
    OverDrive,
    Reverb,
    MCO, // Midi controlled Oscillator
         // PMCO, // pollyphonic Midi Controlled Osc
}

impl Display for ModuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Vco => write!(f, "VCO"),
            Self::Output => write!(f, "Output"),
            Self::Lfo => write!(f, "LFO"),
            Self::Echo => write!(f, "Echo"),
            Self::EnvFilter => write!(f, "Filter"),
            Self::Chorus => write!(f, "Chorus"),
            Self::Delay => write!(f, "Delay"),
            Self::OverDrive => write!(f, "OD"),
            Self::Reverb => write!(f, "Reverb"),
            Self::MCO => write!(f, "MCO"),
            // Self::PMCO => write!(f, "PMCO"),
            // Self:: => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, EnumIter, Hash, PartialEq, Eq)]
pub enum FilterType {
    // None,
    // ADBDR,
    ADSR,
    // AD,
    // OC,
    // TODO: add an AR filter
}

impl Into<Float> for FilterType {
    fn into(self) -> Float {
        match self {
            // Self::None => 1.0,
            // Self::ADBDR => 2.0,
            Self::ADSR => 3.0,
            // Self::OC => 4.0,
            // Self::AD => 5.0,
        }
    }
}

impl From<Float> for FilterType {
    fn from(value: Float) -> Self {
        match value {
            // 1.0..2.0 => Self::None,
            // 2.0..3.0 => Self::ADBDR,
            3.0..4.0 => Self::ADSR,
            // 4.0..5.0 => Self::OC,
            // 5.0..6.0 => Self::AD,
            // _ => Self::None,
            // _ => Self::OC,
            _ => Self::ADSR,
        }
    }
}

pub fn midi_to_freq(midi_note: u8) -> f32 {
    let exp = (f32::from(midi_note) + 36.376_316) / 12.0;

    2.0f32.powf(exp)
}
