use super::Envelope;
use crate::{Float, SAMPLE_RATE};
use anyhow::{bail, Result};

pub const ATTACK_IN: u8 = 3; // sets attack speed in seconds
pub const DECAY_1_IN: u8 = 4; // sets decay 1 speed in seconds
pub const DECAY_THRESHOLD: u8 = 5; // sets the threshold between decay 1 & 2 in amplitude
pub const DECAY_2_IN: u8 = 6; // sets decay 2 speed in seconds

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Phase {
    Neutural,
    Attack,
    Decay1,
    Decay2,
    Release,
}

#[derive(Debug, Clone)]
pub struct Filter {
    phase: Phase,
    env: Float,
    decay_1_speed: Float,
    decay_2_speed: Float,
    attack_speed: Float,
    threshold: Float,
    decay_1: Float,
    decay_2: Float,
    attack: Float,
    release: Float,
    sample_rate: Float,
    pressed: bool,
    release_threshold: Float,
}

impl Filter {
    pub fn new() -> Self {
        // info!("making ADBDR filter");
        let sample_rate = SAMPLE_RATE as Float;
        let attack_speed = 0.0001;
        let attack = 1.0 / (sample_rate * attack_speed);
        let decay_1_speed = 0.1;
        let decay_1 = -1.0 / (sample_rate * decay_1_speed);
        let decay_2_speed = 15.0;
        let decay_2 = -1.0 / (sample_rate * decay_2_speed);

        Self {
            phase: Phase::Neutural,
            env: 0.0,
            decay_1,
            decay_2,
            attack,
            threshold: 0.9,
            sample_rate,
            attack_speed,
            decay_1_speed,
            decay_2_speed,
            release: -1.0 / (sample_rate * 0.01),
            pressed: false,
            release_threshold: 0.05,
        }
    }

    fn set_attack(&mut self, atk_speed: Float) {
        if atk_speed != self.attack_speed {
            self.attack_speed = atk_speed;
            self.attack = 1.0 / (self.sample_rate * atk_speed);
        }
    }

    fn set_decay_1(&mut self, decay_1_speed: Float) {
        if decay_1_speed != self.decay_1_speed {
            self.decay_1_speed = decay_1_speed;
            self.decay_1 = -(1.0 - self.threshold) / (self.sample_rate * decay_1_speed);
        }
    }

    fn set_decay_2(&mut self, decay_2_speed: Float) {
        if decay_2_speed != self.decay_2_speed {
            self.decay_2_speed = decay_2_speed;
            self.decay_2 =
                -(self.threshold - self.release_threshold) / (self.sample_rate * decay_2_speed);
        }
    }

    fn set_threshold(&mut self, threshold: Float) {
        if self.threshold != threshold {
            self.threshold = threshold;
        }
    }

    fn internal_update_phase(&mut self) {
        if self.phase == Phase::Attack && self.env >= 1.0 {
            self.phase = Phase::Decay1;
            self.env = 1.0;
            // info!("changing phase to => {:?}", self.phase);
        } else if self.phase == Phase::Decay1 && self.env <= self.threshold {
            self.phase = Phase::Decay2;
            // info!("changing phase to => {:?}", self.phase);
        } else if self.phase == Phase::Decay2 && self.env <= self.release_threshold {
            self.phase = Phase::Release;
            // info!("changing phase to => {:?}", self.phase);
        } else if self.phase == Phase::Release && self.env <= 0.0 {
            self.phase = Phase::Neutural;
            self.env = 0.0;
            // info!("changing phase to => {:?}", self.phase);
        }
    }
}

impl Envelope for Filter {
    fn get_env(&mut self) -> Float {
        // trace!("ADBDR envelope value {}", self.env);
        self.env
    }

    fn set_env(&mut self, env: Float) {
        self.env = env;
    }

    fn get_step(&mut self) -> Float {
        match self.phase {
            Phase::Attack => self.attack,
            Phase::Decay1 => self.decay_1,
            Phase::Decay2 => self.decay_2,
            Phase::Release => self.release,
            Phase::Neutural => 0.0,
        }
    }

    fn update_phase(&mut self) {
        self.internal_update_phase()
    }

    fn open_filter(&mut self, samples: Vec<Float>) -> bool {
        let sample: Float = samples.iter().sum::<Float>().tanh();
        // info!("envelope filter is open: {}", sample >= 0.75);

        if self.pressed() && sample <= 0.75 {
            // info!("release");
            self.phase = Phase::Release;
            self.pressed = false;
        } else if self.phase == Phase::Neutural && sample >= 0.75 {
            // info!("pressed");
            self.phase = Phase::Attack;
            self.pressed = true;
        }

        self.pressed
    }

    fn take_input(&mut self, input: u8, samples: Vec<Float>) -> Result<()> {
        let sample: Float = samples.iter().sum::<Float>().tanh();

        match input {
            // attack in
            0 => self.set_attack(sample),
            // decay_1 speed in
            1 => self.set_decay_1(sample),
            // decay_threshold in
            2 => self.set_threshold(sample),
            // decay_2 speed in
            3 => self.set_decay_2(sample),
            n => bail!("{n} is not a valid input for the ADBDR filter."),
        }

        Ok(())
    }

    fn pressed(&mut self) -> bool {
        self.phase != Phase::Neutural
    }
}
