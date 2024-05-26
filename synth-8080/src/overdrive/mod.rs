use crate::{common::Module, Float};
use tracing::*;

pub const N_INPUTS: u8 = 2;
pub const N_OUTPUTS: u8 = 1;

pub const AUDIO_INPUT: u8 = 0;
pub const GAIN_INPUT: u8 = 1;

pub const AUDIO_OUT: u8 = 0;

pub struct OverDrive {
    gain: Float,
    audio_in: Float,
}

impl OverDrive {
    pub fn new() -> Self {
        OverDrive {
            gain: (1.0 as Float + 1.1).powi(2),
            audio_in: 0.0,
        }
    }
}

impl Module for OverDrive {
    fn get_samples(&mut self) -> Vec<(u8, Float)> {
        vec![(AUDIO_OUT, (self.audio_in * self.gain).tanh())]
    }

    fn recv_samples(&mut self, input_n: u8, samples: &[Float]) {
        let sample = samples.iter().sum();

        if input_n == AUDIO_INPUT {
            self.audio_in = sample;
        } else if input_n == GAIN_INPUT {
            self.gain = (sample + 1.1).powi(4);
        } else {
            error!("invalid input designation: {input_n} for the OverDrive Modules.");
        }
    }

    fn get_input_names() -> impl Iterator<Item = impl std::fmt::Display> {
        ["Audio In", "Gain"].iter()
    }

    fn get_output_names() -> impl Iterator<Item = impl std::fmt::Display> {
        ["Audio Out"].iter()
    }
}
