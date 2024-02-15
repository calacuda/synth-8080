use super::Osc;
use crate::Float;
use fon::chan::Channel;
use twang::ops::Gain;
use twang::osc::Triangle;

#[derive(Default)]
pub struct Oscilator {
    piano: Triangle,
    frequency: f32,
    volume: f32,
}

impl Oscilator {
    pub fn init(&mut self) {
        self.volume = 1.0;
    }
}

impl Osc for Oscilator {
    fn get_sample(&mut self) -> Float {
        let sample: f32 = Gain
            .step(self.piano.step(self.frequency), self.volume.into())
            .to_f32();

        sample as Float
    }

    fn set_frequency(&mut self, frequency: Float) {
        self.frequency = frequency as f32;
    }
}
