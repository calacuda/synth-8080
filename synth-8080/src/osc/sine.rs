use super::Osc;
use crate::Float;
use fon::chan::Channel;
use tracing::info;
use twang::noise::White;
use twang::ops::Gain;
use twang::osc::Sine;

/// First ten harmonic volumes of a piano sample (sounds like electric piano).
const HARMONICS: [f32; 10] = [
    0.700, 0.243, 0.229, 0.095, 0.139, 0.087, 0.288, 0.199, 0.124, 0.090,
];
const VOLUME: f32 = 0.1;

#[derive(Default)]
pub struct Oscilator {
    // White noise generator.
    white: White,
    // 10 harmonics for 3 pitches.
    piano: [Sine; 10],
    frequency: f32,
    overtones: bool,
    volume: f32,
}

impl Oscilator {
    pub fn init(&mut self) {
        // Adjust phases of harmonics.
        for harmonic in self.piano.iter_mut() {
            harmonic.shift(self.white.step());
        }

        self.volume = 1.0;
        self.overtones = false;
    }
}

impl Osc for Oscilator {
    fn get_sample(&mut self) -> Float {
        let sample: f32 = if self.overtones {
            // info!("overtones => {}", self.volume);
            self.piano
                .iter_mut()
                .enumerate()
                .zip(HARMONICS.iter())
                .map(|((i, o), v)| {
                    // Get next sample from oscillator.
                    let sample = o.step(self.frequency * (i + 1) as f32);
                    // Pan the generated harmonic center
                    Gain.step(sample, (*v * self.volume).into()).to_f32()
                })
                .sum::<f32>()
        } else {
            Gain.step(self.piano[0].step(self.frequency), self.volume.into())
                .to_f32()
        };
        // info!("{sample} => {:?}", sample as Float);

        sample as Float
    }

    fn set_frequency(&mut self, frequency: Float) {
        self.frequency = frequency as f32;
    }

    fn set_overtones(&mut self, on: bool) {
        if on {
            self.volume = VOLUME;
        } else {
            self.volume = 1.0;
        }

        self.overtones = on;
    }
}
