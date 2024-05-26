use crate::{common::bend_range, Float};
pub use lib::OscType;
use std::sync::Arc;
use tracing::trace;

pub mod saw;
pub mod sine;
pub mod square;
pub mod triangle;

pub type Harmonics = Arc<[f32; HARMONICS_SIZE]>;

pub const HARMONICS_SIZE: usize = 15;
// pub const DEFAULT_HARMONICS: [f32; HARMONICS_SIZE] = [
//     0.700, 0.243, 0.229, 0.095, 0.139, 0.087, 0.288, 0.199, 0.124, 0.090,
// ];
// pub const DEFAULT_HARMONICS: [f32; HARMONICS_SIZE] = [
//     1.0,
//     1.0 / 2.0,
//     1.0 / 3.0,
//     1.0 / 4.0,
//     1.0 / 5.0,
//     1.0 / 6.0,
//     1.0 / 7.0,
//     1.0 / 8.0,
//     1.0 / 9.0,
//     1.0 / 10.0,
// ];
// pub const DEFAULT_HARMONICS: [f32; HARMONICS_SIZE] = [
//     1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
// ];
pub const DEFAULT_HARMONICS: [f32; HARMONICS_SIZE] = [
    1.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
    1.0 / 2.0,
];

pub trait Osc: Send {
    // fn get_sample(&mut self) -> Float;
    fn get_sample(&mut self) -> Float {
        if self.overtones() {
            self.overtones_sample()
        } else {
            self.puretone_sample()
        }
    }

    /// sets the frequency of the desired note
    fn set_frequency(&mut self, frequency: Float);

    /// enables or disabled overtone generation
    fn enable_overtones(&mut self, enabled: bool);

    /// returns true if overtones are enabled
    fn overtones(&mut self) -> bool;

    /// generate a sample in over tones mode
    fn overtones_sample(&mut self) -> Float;

    /// generate a sample when not in overtone mode
    fn puretone_sample(&mut self) -> Float;
}

pub struct Oscillator {
    pub osc: Box<dyn Osc>,
    pub waveform: OscType,
    pub overtones: bool,
    pub frequency: Float,
    pub bend: Float,
    pub volume: Float,
}

impl Oscillator {
    pub fn new() -> Self {
        // trace!("making a sine wave oscillator");
        let mut osc = Box::new(sine::Oscillator::default());
        // trace!("made a sine wave oscillator");
        osc.init();
        osc.enable_overtones(true);
        let waveform = OscType::Sine;
        let overtones = true;
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
        self.osc.get_sample() * self.volume * (1.0 / (HARMONICS_SIZE as Float).sqrt())
    }

    pub fn set_waveform(&mut self, waveform: OscType) {
        self.osc = match waveform {
            OscType::Sine => {
                let mut osc = sine::Oscillator::default();
                osc.init();
                Box::new(osc)
            }
            OscType::Square => {
                let mut osc = square::Oscillator::default();
                osc.init();
                Box::new(osc)
            }
            OscType::Triangle => {
                let mut osc = triangle::Oscillator::default();
                osc.init();
                Box::new(osc)
            }
            OscType::SawTooth => {
                let mut osc = saw::Oscillator::default();
                osc.init();
                Box::new(osc)
            }
        };

        self.osc.enable_overtones(true);

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

    pub fn enable_overtones(&mut self, enabled: bool) {
        self.overtones = enabled;
        self.osc.enable_overtones(enabled);
    }
}
