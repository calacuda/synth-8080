use crate::Float;
use serde::Deserialize;

pub mod sin_wt;

#[derive(Deserialize, Debug, Clone)]
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

pub trait Osc: Send {
    fn get_sample(&mut self) -> Float;

    fn set_frequency(&mut self, frequency: Float);
}
