use crate::{common::bend_range, Float};
pub use lib::OscType;

pub mod saw;
pub mod sine;
pub mod square;
pub mod triangle;

pub trait Osc: Send {
    fn get_sample(&mut self) -> Float;

    fn set_frequency(&mut self, frequency: Float);
}

pub struct Oscilator {
    pub osc: Box<dyn Osc>,
    pub waveform: OscType,
    pub overtones: bool,
    pub frequency: Float,
    pub bend: Float,
    pub volume: Float,
}

impl Oscilator {
    pub fn new() -> Self {
        let mut osc = Box::new(sine::Oscilator::default());
        osc.init();
        let waveform = OscType::Sine;
        let overtones = false;
        let frequency = 0.0;
        let bend = bend_range();
        let volume = 1.0;

        Self {
            osc,
            waveform,
            overtones,
            frequency,
            bend,
            volume,
        }
    }

    pub fn set_frequency(&mut self, frequency: Float) {
        self.frequency = frequency;
        self.osc.set_frequency(frequency);
    }

    pub fn get_sample(&mut self) -> Float {
        self.osc.get_sample() * self.volume
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
