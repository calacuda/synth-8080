use crate::{common::Module, Float, SAMPLE_RATE};
use tracing::*;

pub const N_INPUTS: u8 = 3;
pub const N_OUTPUTS: u8 = 1;

pub const AUDIO_INPUT: u8 = 0;
pub const SPEED_INPUT: u8 = 1;
pub const DECAY_INPUT: u8 = 2;

pub struct Buff {
    pub size: usize,
    pub buff: [Float; SAMPLE_RATE as usize],
    pub i: usize,
    pub step: usize,
    pub volume: Float,
}

impl Buff {
    pub fn get_sample(&mut self, input_sample: Float) -> Float {
        let echo =
            ((self.buff[(self.i + self.step) % self.size] * self.volume) + input_sample).tanh();
        self.i = (self.i + 1) % self.size;
        self.buff[self.i] = echo;
        echo
    }

    /// sets speed, takes speed in seconds
    pub fn set_speed(&mut self, speed: Float) {
        // info!("speed: {}", speed);
        self.step = (SAMPLE_RATE as Float * speed) as usize;
        // info!("step:  {}", self.step);
    }

    pub fn set_volume(&mut self, volume: Float) {
        self.volume = volume;
    }
}

pub struct Echo {
    // pub routing_table: Router,
    buff: Buff,
    /// where the data from the audio input is stored
    audio_in: Float,
    _id: u8,
}

impl Echo {
    pub fn new(_id: u8) -> Self {
        const BUFF_SIZE: usize = SAMPLE_RATE as usize;

        let mut buff = Buff {
            size: BUFF_SIZE,
            buff: [0.0; BUFF_SIZE],
            i: 0,
            step: 0,
            volume: 0.9,
        };
        let audio_in = 0.0;

        // buff.set_speed(0.4);
        buff.set_speed(0.4);

        Self {
            buff,
            audio_in,
            _id,
        }
    }
}

impl Module for Echo {
    fn get_samples(&mut self) -> Vec<(u8, Float)> {
        vec![(0, self.buff.get_sample(self.audio_in))]
    }

    fn recv_samples(&mut self, input_n: u8, samples: &[Float]) {
        let sample: Float = samples.iter().sum();

        if input_n == AUDIO_INPUT {
            self.audio_in = sample.tanh();
        } else if input_n == SPEED_INPUT {
            self.buff.set_speed((sample.tanh() + 1.0) * 0.5);
        } else if input_n == DECAY_INPUT {
            self.buff.set_volume(sample.tanh());
        } else {
            error!("invalid input for echo module: {input_n}");
        }
    }

    fn get_input_names() -> impl Iterator<Item = impl std::fmt::Display> {
        ["Audio In", "Speed", "Vol."].iter()
    }

    fn get_output_names() -> impl Iterator<Item = impl std::fmt::Display> {
        ["Audio Out"].iter()
    }
}
