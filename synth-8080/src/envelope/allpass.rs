use super::Filter;
use lib::{Float, SAMPLE_RATE};
use std::f64::consts::PI;
use tracing::trace;

pub struct AllPassFilter {
    cutoff: Float,
    base_cutoff: Float,
    dn_1: Float,
    env: Float,
    a1: Float,
    highpass: bool,
    base_resonance: Float,
    resonance: Float,
}

impl AllPassFilter {
    pub fn new() -> Self {
        let cutoff = 62.0;
        Self {
            cutoff,
            base_cutoff: cutoff,
            dn_1: 0.0,
            env: 0.0,
            a1: 0.0,
            highpass: false,
            base_resonance: 1.0,
            resonance: 1.0,
        }
    }

    fn set_a1_coef(&mut self) {
        let tan = (PI as Float * self.cutoff / SAMPLE_RATE as Float).tan();
        self.a1 = (tan - 1.0) / (tan + 1.0)
    }

    pub fn wiggle_cutoff(&mut self, wiggle_amount: Float) {
        self.cutoff = self.base_cutoff + (10_000.0 * wiggle_amount);
        // trace!(
        //     "base_cutoff: {} | cutoff: {}, | wiggle: {}",
        //     self.base_cutoff,
        //     self.cutoff,
        //     wiggle_amount
        // );

        self.set_a1_coef();
    }

    pub fn wiggle_resonance(&mut self, wiggle_amount: Float) {
        self.resonance = self.base_resonance + (self.base_resonance * wiggle_amount);
    }
}

impl Filter for AllPassFilter {
    fn init(&mut self) {
        self.set_a1_coef();
    }

    fn take_env(&mut self, env: Float) {
        self.env = env;
        // TODO: do stuff
    }

    fn get_sample(&mut self, audio_in: Float) -> Float {
        let result = self.a1 * audio_in + (self.dn_1 * self.resonance);
        self.dn_1 = audio_in - self.a1 * (result * self.resonance);
        // self.dn_1 = audio_in - self.a1 * result;

        if self.highpass {
            result * -0.5
        } else {
            result * 0.5
        }
    }

    /// takes a number between 0 and 1.0
    fn set_cutoff(&mut self, cutoff: Float) {
        // trace!("input cutoff: {cutoff}");
        let cutoff = (cutoff * 10_000.0);
        self.cutoff = cutoff;
        self.base_cutoff = cutoff;
        // trace!("set cutoff to: {}", self.cutoff);
        self.set_a1_coef();
    }

    fn set_resonance(&mut self, resonance: Float) {
        self.resonance = resonance;
        self.base_resonance = resonance;
    }
}
