use crate::{common::Module, Float};
use reverb::Reverb;
use tracing::*;

pub const N_INPUTS: u8 = 3;
pub const N_OUTPUTS: u8 = 1;

pub const AUDIO_INPUT: u8 = 0;
pub const GAIN_INPUT: u8 = 1;
pub const DECAY_INPUT: u8 = 2;

pub struct ReverbModule {
    verb: Reverb,
    audio_in: Float,
    gain: f32,
}

impl ReverbModule {
    pub fn new() -> Self {
        ReverbModule {
            verb: Reverb::new()
                .diffusion(0.75, 0.75, 0.75, 0.75)
                .decay(0.5)
                .clone(),
            audio_in: 0.0,
            gain: 1.0,
        }
    }
}

impl Module for ReverbModule {
    fn get_samples(&mut self) -> Vec<(u8, Float)> {
        vec![(
            0,
            self.verb.calc_sample(self.audio_in as f32, self.gain) as Float,
        )]
    }

    fn recv_samples(&mut self, input_n: u8, samples: &[Float]) {
        let sample: Float = samples.iter().sum();

        if input_n == AUDIO_INPUT {
            self.audio_in = sample;
        } else if input_n == GAIN_INPUT {
            self.gain = sample as f32;
        } else if input_n == DECAY_INPUT {
            self.verb = self.verb.decay((sample as f32 + 1.0) * 0.5).clone();
        } else {
            error!("invalid input: {input_n}, to reverb");
        }
    }
}
