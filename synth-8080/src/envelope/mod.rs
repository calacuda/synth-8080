use crate::{common::Module, Float};
use anyhow::Result;
use lib::FilterType;
use tracing::*;

pub mod ad;
pub mod adbdr;
pub mod adsr;
pub mod none;
pub mod oc;

pub const N_INPUTS: u8 = 7;
pub const N_OUTPUTS: u8 = 1;

pub const FILTER_SELECT_IN: u8 = 0;
pub const AUDIO_IN: u8 = 1;
pub const FILTER_OPEN_IN: u8 = 2;

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
}

impl EnvelopeFilter {
    pub fn new(id: u8) -> Self {
        Self {
            filter_type: FilterType::ADBDR,
            pressed: false,
            envelope: Box::new(adbdr::Filter::new()),
            audio_in: 0.0,
            id,
        }
    }

    pub fn set_filter_type(&mut self, filter_type: FilterType) {
        self.filter_type = filter_type;
        info!("setting filter type to {:?}", self.filter_type);
        self.envelope = match self.filter_type {
            // FilterType::None => Box::new(none::Filter::new()),
            FilterType::ADSR => Box::new(adsr::Filter::new()),
            FilterType::ADBDR => Box::new(adbdr::Filter::new()),
            FilterType::OC => Box::new(oc::Filter::new()),
            FilterType::AD => Box::new(ad::Filter::new()),
        };
    }

    pub fn is_pressed(&mut self) -> bool {
        self.envelope.pressed()
    }
}

impl Module for EnvelopeFilter {
    fn get_samples(&mut self) -> Vec<(u8, Float)> {
        vec![(0, self.audio_in * self.envelope.step())]
    }

    fn recv_samples(&mut self, input_n: u8, samples: &[Float]) {
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
            let _ = self.envelope.take_input(0, samples.to_vec());
        } else if input_n == 4 {
            let _ = self.envelope.take_input(1, samples.to_vec());
        } else if input_n == 5 {
            let _ = self.envelope.take_input(2, samples.to_vec());
        } else if input_n == 6 {
            let _ = self.envelope.take_input(3, samples.to_vec());
        } else {
            error!("invalid input selection {:?}:{input_n}", self.filter_type);
        }
    }
}
