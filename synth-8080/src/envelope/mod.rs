use std::f32::NAN;

use crate::{common::Module, Float};
use anyhow::Result;
use lib::FilterType;
use log::info;
use tracing::*;

use self::allpass::{AllPassFilter, LowPassFilter};

pub mod ad;
pub mod adbdr;
pub mod adsr;
pub mod allpass;
pub mod none;
pub mod oc;

pub const N_INPUTS: u8 = 8;
pub const N_OUTPUTS: u8 = 3;

pub const FILTER_SELECT_IN: u8 = 0;
pub const AUDIO_IN: u8 = 1;
pub const FILTER_OPEN_IN: u8 = 2;
pub const AUDIO_OUT: u8 = 0;

pub trait Envelope: Send {
    fn step(&mut self) -> Float {
        self.step_env();
        let env = self.get_env();
        self.update_phase();

        env
    }

    /// returns the current envelope value
    fn get_env(&mut self) -> Float;

    fn set_env(&mut self, env: Float);

    fn step_env(&mut self) {
        let new_env = self.get_env() + self.get_step();
        self.set_env(new_env);
    }

    /// gets the step amount based on phase
    fn get_step(&mut self) -> Float;

    /// stpes the phase (shifts to the next phase if the conditions are right)
    fn update_phase(&mut self);

    /// takes input address and samples, handles adjusting values acouringly, will return error if
    /// the input doesn't exist for the current filter
    fn take_input(&mut self, input: u8, samples: Vec<Float>) -> Result<()>;

    /// opens or closses the filter depending on the sum of `samples`. returns whether the filter is
    /// pressed.
    fn open_filter(&mut self, samples: Vec<Float>) -> bool;

    /// returns true if the filter is not in its neuteral state.
    fn pressed(&mut self) -> bool;
}

pub trait Filter: Send {
    fn init(&mut self);

    /// takes an env value from envelope and updates internal values acourdingly.
    fn take_env(&mut self, env: Float);

    /// returns generated sample.
    fn get_sample(&mut self, audio_in: Float) -> Float;

    /// sets cutoff_frequency
    fn set_cutoff(&mut self, cutoff: Float);

    /// sets resonece
    fn set_resonance(&mut self, resonance: Float);
}

pub struct EnvelopeFilter {
    /// which filter is currently in use
    pub filter_type: FilterType,
    /// stores if the envelope is pressed
    pub pressed: bool,
    /// the filter that is currently in use
    pub envelope: Box<dyn Envelope>,
    /// stores the audio input sample
    pub audio_in: Float,
    /// the id which identifies this module from all others
    pub id: u8,
    // pub allpass: AllPassFilter,
    // pub lowpass: LowPassFilter,
    pub filter: Box<dyn Filter>,
}

impl EnvelopeFilter {
    pub fn new(id: u8) -> Self {
        let filter: Box<dyn Filter> = if cfg!(feature = "allpass") {
            warn!("using allpass-based lowpass filter to save reasource");
            let mut filter = AllPassFilter::new();
            filter.init();
            Box::new(filter)
        } else {
            warn!("using true lowpass filter to for better quality");
            Box::new(LowPassFilter::new())
        };

        Self {
            filter_type: FilterType::ADSR,
            pressed: false,
            envelope: Box::new(adsr::Filter::new()),
            audio_in: 0.0,
            id,
            // allpass: filter,
            // lowpass: LowPassFilter::new(),
            filter,
        }
    }

    pub fn set_filter_type(&mut self, filter_type: FilterType) {
        self.filter_type = filter_type;
        info!("setting filter type to {:?}", self.filter_type);
        self.envelope = match self.filter_type {
            // FilterType::None => Box::new(none::Filter::new()),
            FilterType::ADSR => Box::new(adsr::Filter::new()),
            // FilterType::ADBDR => Box::new(adbdr::Filter::new()),
            // FilterType::OC => Box::new(oc::Filter::new()),
            // FilterType::AD => Box::new(ad::Filter::new()),
        };
    }

    pub fn is_pressed(&mut self) -> bool {
        self.envelope.pressed()
    }
}

impl Module for EnvelopeFilter {
    fn get_samples(&mut self) -> Vec<(u8, Float)> {
        let env = self.envelope.step();
        self.filter.take_env(env);
        let sample = (self.audio_in + self.filter.get_sample(self.audio_in)) * env;
        // let sample = self.audio_in * env;

        let open = if self.envelope.pressed() { 1.0 } else { 0.0 };

        vec![(0, sample), (1, env), (2, open)]
    }

    fn recv_samples(&mut self, input_n: u8, samples: &[Float]) {
        // TODO: add lowpass filter controls
        if input_n == FILTER_SELECT_IN {
            // self.filter_select_in_cons.lock().unwrap().push(connection);
            let input = samples.iter().sum::<Float>().tanh();
            if input > 1.0 {
                // let mut ft = ft.lock().unwrap();
                self.set_filter_type(input.into());
            }
        } else if input_n == AUDIO_IN {
            let audio = samples.iter().sum::<Float>().tanh();
            self.audio_in = audio
        } else if input_n == FILTER_OPEN_IN {
            // let input: Float = samples.iter().sum();
            self.pressed = self.envelope.open_filter(samples.to_vec());
            // info!("pressed => {}", self.pressed);
        } else if input_n == 3 {
            let sample: Float = samples.iter().sum();
            // self.allpass.set_cutoff(sample.tanh());
            // self.allpass.wiggle_cutoff(sample.tanh());
            // self.filter.
        } else if input_n == 4 {
            let _ = self.envelope.take_input(0, samples.to_vec());
        } else if input_n == 5 {
            let _ = self.envelope.take_input(1, samples.to_vec());
        } else if input_n == 6 {
            let _ = self.envelope.take_input(2, samples.to_vec());
        } else if input_n == 7 {
            let _ = self.envelope.take_input(3, samples.to_vec());
        } else {
            error!("invalid input selection {:?}:{input_n}", self.filter_type);
        }
    }

    fn get_input_names() -> impl Iterator<Item = impl std::fmt::Display> {
        [
            "Filter Select",
            "Audio In",
            "Open Filter",
            "Wiggle Resonace",
            "Attack",
            "Decay",
            "Sus/Break",
            "Decay2",
        ]
        .iter()
    }

    fn get_output_names() -> impl Iterator<Item = impl std::fmt::Display> {
        ["Audio Out", "Env", "Open"].iter()
    }
}
