use serde::{Deserialize, Serialize};

pub mod communication;
pub mod notes;

pub type Float = f32;
pub type ModuleId = u8;
// pub const SAMPLE_RATE: u32 = 48_000;
// pub const SAMPLE_RATE: u32 = 44_100;
// pub const SAMPLE_RATE: u32 = 22_050;
// pub const SAMPLE_RATE: u32 = 16_000;
pub const SAMPLE_RATE: u32 = 24_000;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscType {
    #[serde(rename = "sine", alias = "sin")]
    Sine,
    #[serde(rename = "square", alias = "squ")]
    Square,
    #[serde(rename = "triangle", alias = "tri")]
    Triangle,
    #[serde(rename = "saw-tooth", alias = "sawtooth", alias = "saw")]
    SawTooth,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Connection {
    pub src_module: u8,
    pub src_output: u8,
    pub dest_module: u8,
    pub dest_input: u8,
}
