#![feature(exclusive_range_pattern)]
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter, EnumString};

pub mod communication;
pub mod notes;

#[cfg(not(feature = "HiFi"))]
pub type Float = f32;
pub type ModuleId = u8;
// pub const SAMPLE_RATE: u32 = 48_000;
// pub const SAMPLE_RATE: u32 = 44_100;
// pub const SAMPLE_RATE: u32 = 22_050;
// pub const SAMPLE_RATE: u32 = 16_000;
#[cfg(not(feature = "HiFi"))]
pub const SAMPLE_RATE: u32 = 24_000;
#[cfg(feature = "HiFi")]
pub type Float = f64;
#[cfg(feature = "HiFi")]
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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, EnumIter, Hash, PartialEq, Eq)]
pub enum FilterType {
    // None,
    ADBDR,
    ADSR,
    AD,
    OC,
    // TODO: add an AR filter
}

impl Into<Float> for FilterType {
    fn into(self) -> Float {
        match self {
            // Self::None => 1.0,
            Self::ADBDR => 2.0,
            Self::ADSR => 3.0,
            Self::OC => 4.0,
            Self::AD => 5.0,
        }
    }
}

impl From<Float> for FilterType {
    fn from(value: Float) -> Self {
        match value {
            // 1.0..2.0 => Self::None,
            2.0..3.0 => Self::ADBDR,
            3.0..4.0 => Self::ADSR,
            4.0..5.0 => Self::OC,
            5.0..6.0 => Self::AD,
            // _ => Self::None,
            _ => Self::OC,
        }
    }
}
