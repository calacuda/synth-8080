use super::{Harmonics, Osc, DEFAULT_HARMONICS, HARMONICS_SIZE};
use crate::Float;
use fon::chan::Channel;
use twang::noise::White;
use twang::ops::Gain;
use twang::osc::Sawtooth;

#[derive(Clone, Debug, Default)]
pub struct Oscillator {
    piano: [Sawtooth; HARMONICS_SIZE],
    frequency: f32,
    volume: f32,
    overtones: bool,
    harmonics: Harmonics,
}

// impl Default for Oscillator {
//     fn default() -> Self {
//         Self {
//             harmonics: DEFAULT_HARMONICS.into(),
//             ..Default::default()
//         }
//     }
// }

impl Oscillator {
    pub fn init(&mut self) {
        self.volume = 1.0;
        self.harmonics = DEFAULT_HARMONICS.into();

        let mut white_noise = White::default();

        // Adjust phases of harmonics.
        for harmonic in self.piano.iter_mut() {
            harmonic.shift(white_noise.step());
        }
    }

    // fn puretone_sample(&mut self) -> Float {
    //     let sample: f32 = Gain
    //         .step(self.piano[0].step(self.frequency), self.volume.into())
    //         .to_f32();
    //
    //     sample as Float
    // }

    // fn overtones_sample(&mut self) -> Float {
    //     // self.puretone_sample()
    //     let sample: f32 = self
    //         .piano
    //         .iter_mut()
    //         .enumerate()
    //         .zip(self.harmonics.iter())
    //         .map(|((i, o), v)| {
    //             // Get next sample from oscillator.
    //             let sample = o.step(self.frequency * (i + 1) as f32);
    //             // Pan the generated harmonic center
    //             Gain.step(sample, (v * self.volume).into()).to_f32()
    //         })
    //         .sum();
    //     sample as Float
    // }
}

impl Osc for Oscillator {
    // fn get_sample(&mut self) -> Float {
    //     if self.overtones {
    //         self.overtones_sample()
    //     } else {
    //         self.puretone_sample()
    //     }
    // }

    fn overtones_sample(&mut self) -> Float {
        // self.puretone_sample()
        let sample: f32 = self
            .piano
            .iter_mut()
            .enumerate()
            .zip(self.harmonics.iter())
            .map(|((i, o), v)| {
                // Get next sample from oscillator.
                let sample = o.step(self.frequency * (i + 1) as f32);
                // Pan the generated harmonic center
                Gain.step(sample, (v * self.volume).into()).to_f32()
            })
            .sum();
        sample as Float
    }

    fn puretone_sample(&mut self) -> Float {
        let sample: f32 = Gain
            .step(self.piano[0].step(self.frequency), self.volume.into())
            .to_f32();

        sample as Float
    }

    fn set_frequency(&mut self, frequency: Float) {
        self.frequency = frequency as f32;
    }

    fn enable_overtones(&mut self, enabled: bool) {
        self.overtones = enabled;
    }

    fn overtones(&mut self) -> bool {
        self.overtones
    }
}
