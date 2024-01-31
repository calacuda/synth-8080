use crate::Float;
use crate::SAMPLE_RATE;
use std::f64::consts::PI;

pub const TABLE_SIZE: usize = 128;

#[derive(Clone, Debug, Copy)]
pub struct WavetableOscillator {
    sample_rate: u32,
    wave_table: [Float; TABLE_SIZE],
    index: Float,
    index_increment: Float,
}

impl WavetableOscillator {
    pub fn new() -> Self {
        let two_pi = PI * 2.0;

        let mut wave_table = [0.0; TABLE_SIZE];
        (0..TABLE_SIZE).for_each(|n| {
            wave_table[n] = (two_pi as Float * n as Float / TABLE_SIZE as Float).sin()
        });

        Self {
            sample_rate: SAMPLE_RATE,
            wave_table,
            index: 0.0,
            index_increment: 0.0,
        }
    }

    pub fn set_freq(&mut self, frequency: Float) {
        self.index_increment = frequency * TABLE_SIZE as Float / self.sample_rate as Float;
    }

    pub fn sample(&mut self) -> Float {
        let sample = self.lerp();
        self.index += self.index_increment;
        self.index %= TABLE_SIZE as Float;
        sample
    }

    fn lerp(&self) -> Float {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % TABLE_SIZE;

        let next_index_weight = self.index - truncated_index as Float;
        let truncated_index_weight = 1.0 - next_index_weight;

        truncated_index_weight * self.wave_table[truncated_index]
            + next_index_weight * self.wave_table[next_index]
    }
}

impl super::Osc for WavetableOscillator {
    fn get_sample(&mut self) -> Float {
        // let sample = self.sample();
        // println!("{sample}");
        // sample
        self.sample()
    }

    fn set_frequency(&mut self, frequency: Float) {
        self.set_freq(frequency)
    }
}

// impl Iterator for WavetableOscillator {
//     type Item = Float;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         Some(self.sample())
//     }
// }
