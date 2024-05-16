use super::Envelope;
use crate::{Float, SAMPLE_RATE};
use anyhow::{bail, Result};

pub const N_INPUTS: u8 = 4; // three for all filters, one for this filter
pub const N_OUTPUTS: u8 = 1;

pub const SPEED_IN: u8 = 3; // sets attack speed in seconds

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Phase {
    Neutural,
    Attack,
    Decay,
    Release,
}

#[derive(Debug, Clone)]
pub struct Filter {
    phase: Phase,
    env: Float,
    pub attack_speed: Float,
    pub attack: Float,
    sample_rate: Float,
    // release: Float,
    pub pressed: bool,
}

impl Filter {
    pub fn new() -> Self {
        let sample_rate = SAMPLE_RATE as Float;
        let attack_speed = 0.5;
        let attack = 1.0 / (sample_rate * attack_speed);

        Self {
            // pressed: false,
            phase: Phase::Neutural,
            // i: 0,
            env: 0.0,
            attack,
            sample_rate,
            attack_speed,
            // release: -1.0 / (sample_rate * 0.01),
            pressed: false,
        }
    }

    fn set_speed(&mut self, atk_speed: Float) {
        if atk_speed != self.attack_speed {
            self.attack_speed = atk_speed;
            self.attack = 1.0 / (self.sample_rate * atk_speed);
        }
    }

    fn internal_update_phase(&mut self) {
        if self.phase == Phase::Attack && self.env >= 1.0 {
            self.phase = Phase::Decay;
            self.env = 1.0;
            // info!("chaning phase to => {:?}", self.phase);
        } else if self.phase == Phase::Decay && self.env <= 0.0 {
            self.phase = Phase::Release;
            // info!("chaning phase to => {:?}", self.phase);
        } else if self.phase == Phase::Release && self.env <= 0.0 {
            self.phase = Phase::Neutural;
            self.env = 0.0;
            // info!("chaning phase to => {:?}", self.phase);
        }
    }
}

impl Envelope for Filter {
    fn get_env(&mut self) -> Float {
        self.env
    }

    fn set_env(&mut self, env: Float) {
        self.env = env;
    }

    fn get_step(&mut self) -> Float {
        match self.phase {
            Phase::Attack => self.attack,
            Phase::Decay => -self.attack,
            Phase::Release => -self.attack,
            Phase::Neutural => 0.0,
        }
    }

    fn update_phase(&mut self) {
        self.internal_update_phase()
    }

    fn open_filter(&mut self, samples: Vec<Float>) -> bool {
        let sample: Float = samples.iter().sum::<Float>().tanh();

        if self.pressed && sample <= 0.75 {
            // info!("release");
            self.phase = Phase::Release;
            self.pressed = false;
        } else if !self.pressed && self.phase == Phase::Neutural && sample >= 0.75 {
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
            SPEED_IN => self.set_speed(sample),
            n => bail!("{n} is not a valid input for the ADBDR filter."),
        }

        Ok(())
    }

    fn pressed(&mut self) -> bool {
        self.phase != Phase::Neutural
    }
}
