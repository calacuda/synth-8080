use super::Filter;
use lib::{Float, SAMPLE_RATE};
use std::f64::consts::PI;
use synfx_dsp::{Biquad, BiquadCoefs};
use tracing::{error, info, trace};

pub struct AllPassFilter {
    cutoff: Float,
    base_cutoff: Float,
    dn_1: Float,
    env: Float,
    a1: Float,
    highpass: bool,
    base_resonance: Float,
    resonance: Float,
    wiggle_discount: Float,
}

impl AllPassFilter {
    pub fn new() -> Self {
        let cutoff = 9_950.0;
        Self {
            cutoff,
            base_cutoff: cutoff,
            dn_1: 0.0,
            env: 0.0,
            a1: 0.0,
            highpass: false,
            base_resonance: 1.0,
            resonance: 0.5,
            wiggle_discount: 0.2,
        }
    }

    fn set_a1_coef(&mut self) {
        let tan = (PI as Float * self.cutoff / SAMPLE_RATE as Float).tan();
        self.a1 = (tan - 1.0) / (tan + 1.0)
    }

    pub fn wiggle_cutoff(&mut self, wiggle_amount: Float) {
        self.cutoff = self.base_cutoff + (12_000.0 * wiggle_amount) + 100.0;
        // trace!(
        //     "base_cutoff: {} | cutoff: {}, | wiggle: {}",
        //     self.base_cutoff,
        //     self.cutoff,
        //     wiggle_amount
        // );

        self.set_a1_coef();
    }

    pub fn wiggle_resonance(&mut self, wiggle_amount: Float) {
        self.resonance = self.base_resonance * wiggle_amount;

        if self.resonance > 1.0 {
            error!("resonance saved from clipping");
            self.resonance = 1.0;
        } else {
            // trace!("resonance => {}", self.resonance);
        }
    }
}

impl Filter for AllPassFilter {
    fn init(&mut self) {
        self.set_a1_coef();
    }

    fn take_env(&mut self, env: Float) {
        self.env = env;
        // self.wiggle_resonance(env);
        // self.wiggle_cutoff(env * 0.25);
    }

    fn get_sample(&mut self, audio_in: Float) -> Float {
        let result = self.a1 * audio_in + self.dn_1 * self.resonance;
        // self.dn_1 = audio_in - self.a1 * (result * self.resonance);
        self.dn_1 = audio_in - self.a1 * result;

        (if self.highpass {
            result * -0.5
        } else {
            result * 0.5
        })
        .tanh()
    }

    /// takes a number between 0 and 1.0
    fn set_cutoff(&mut self, cutoff: Float) {
        // trace!("input cutoff: {cutoff}");
        let cutoff = (cutoff * 12_000.0) + 100.0;
        self.cutoff = cutoff;
        self.base_cutoff = cutoff;
        // trace!("set cutoff to: {}", self.cutoff);
        self.set_a1_coef();
    }

    fn set_resonance(&mut self, resonance: Float) {
        let res = resonance;

        self.resonance = res;
        self.base_resonance = res;
    }
}

pub struct LowPassFilter {
    cutoff: Float,
    base_cutoff: Float,
    resonance: Float,
    base_resonance: Float,
    env: Float,
    filter: Biquad,
}

impl LowPassFilter {
    pub fn new() -> Self {
        let low_pass_settings = BiquadCoefs::lowpass(SAMPLE_RATE as Float, 0.5, 6_000.0);
        let mut filter = Biquad::new();
        filter.set_coefs(low_pass_settings);

        Self {
            cutoff: 6_000.0,
            base_cutoff: 6_000.0,
            resonance: 0.5,
            base_resonance: 0.5,
            env: 0.0,
            filter,
        }
    }

    pub fn wiggle_cutoff(&mut self, wiggle_amount: Float) {
        self.cutoff = self.base_cutoff + (10_000.0 * wiggle_amount);
        // trace!(
        //     "base_cutoff: {} | cutoff: {}, | wiggle: {}",
        //     self.base_cutoff,
        //     self.cutoff,
        //     wiggle_amount
        // );

        self.recalculate();
    }

    fn recalculate(&mut self) {
        // info!(
        //     "using cutoff: {}, and resonance: {}.",
        //     self.cutoff, self.resonance
        // );
        let low_pass_settings =
            BiquadCoefs::lowpass(SAMPLE_RATE as Float, self.resonance, self.cutoff);
        let mut filter = Biquad::new();
        filter.set_coefs(low_pass_settings);

        self.filter = filter;
    }
}

impl Filter for LowPassFilter {
    fn init(&mut self) {}

    fn take_env(&mut self, env: Float) {
        self.env = env;
        // maybe change to minus
        // self.resonance = (env * 2.0 - 1.0);
        // self.wiggle_cutoff(env);
        // info!("resonance => {}", self.resonance);
        // self.recalculate();
    }

    fn get_sample(&mut self, audio_in: Float) -> Float {
        self.filter.tick(audio_in as f32) as Float
    }

    fn set_cutoff(&mut self, cutoff: Float) {
        let cutoff = (cutoff * 12_000.0);

        self.cutoff = cutoff;
        self.base_cutoff = cutoff;
        self.recalculate();
    }

    fn set_resonance(&mut self, resonance: Float) {
        // let scaler = 19_950.0;
        let res = resonance;
        // info!("{resonance} => {res}");

        self.resonance = res;
        self.base_resonance = res;
        self.recalculate();
    }
}
