use crate::{common::bend_range, Float};
use serde::Deserialize;

pub mod saw;
pub mod sine;
pub mod square;
pub mod triangle;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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

    fn set_overtones(&mut self, on: bool);
}

pub struct Oscilator {
    pub osc: Box<dyn Osc>,
    pub waveform: OscType,
    pub overtones: bool,
    pub frequency: Float,
    pub bend: Float,
}

impl Oscilator {
    pub fn new() -> Self {
        let mut osc = Box::new(sine::Oscilator::default());
        osc.init();
        let waveform = OscType::Sine;
        let overtones = false;
        let frequency = 0.0;
        let bend = bend_range();

        Self {
            osc,
            waveform,
            overtones,
            frequency,
            bend,
        }
    }

    pub fn set_frequency(&mut self, frequency: Float) {
        self.frequency = frequency;
        self.osc.set_frequency(frequency);
    }

    pub fn get_sample(&mut self) -> Float {
        self.osc.get_sample()
    }

    pub fn set_waveform(&mut self, waveform: OscType) {
        self.osc = match waveform {
            OscType::Sine => {
                let mut osc = sine::Oscilator::default();
                osc.init();
                Box::new(osc)
            }
            OscType::Square => {
                let mut osc = square::Oscilator::default();
                osc.init();
                Box::new(osc)
            }
            OscType::Triangle => {
                let mut osc = triangle::Oscilator::default();
                osc.init();
                Box::new(osc)
            }
            OscType::SawTooth => {
                let mut osc = saw::Oscilator::default();
                osc.init();
                Box::new(osc)
            }
        };

        self.osc.set_frequency(self.frequency);
        self.osc.set_overtones(self.overtones);
    }

    pub fn set_overtones(&mut self, overtones: bool) {
        self.overtones = overtones;
        self.osc.set_overtones(overtones);
    }

    /// applies a pitch bend by changing the oscilators frequency
    pub fn apply_bend(&mut self, bend: Float) {
        let note = self.frequency;

        let new_note = if bend > 0.0 {
            let shift = note * self.bend;
            note + bend * (shift - note)
        } else if bend < 0.0 {
            let shift = note / self.bend;
            note + bend * (note - shift)
        } else {
            note
        };

        // info!("{new_note}");

        self.osc.set_frequency(new_note);
    }
}
