use crate::{
    common::{Connection, Module},
    Float,
};
use anyhow::Result;
use std::sync::{Arc, Mutex};
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

    /// gets the step ammount based on phase
    fn get_step(&mut self) -> Float;

    /// stpes the phase (shifts to the next phase if the conditions are right)
    fn update_phase(&mut self);

    /// takes input address and samples, handles adjusting values acouringly, will return error if
    /// the input doesn't exist for the current filter
    fn take_input(&mut self, input: u8, samples: Vec<Float>) -> Result<()>;

    fn open_filter(&mut self, samples: Vec<Float>);
}

#[derive(Debug)]
pub enum FilterType {
    None,
    ADBDR,
    ADSR,
    OC,
    // AD,
}

impl Into<Float> for FilterType {
    fn into(self) -> Float {
        match self {
            Self::None => 1.0,
            Self::ADBDR => 2.0,
            Self::ADSR => 3.0,
            Self::OC => 4.0,
        }
    }
}

impl From<Float> for FilterType {
    fn from(value: Float) -> Self {
        match value {
            1.0..2.0 => Self::None,
            2.0..3.0 => Self::ADBDR,
            3.0..4.0 => Self::ADSR,
            4.0..5.0 => Self::OC,
            _ => Self::None,
        }
    }
}

pub struct EnvelopeFilter {
    /// which filter is currently in use
    pub filter_type: FilterType,
    /// where to send the audio that gets generated
    pub outputs: Arc<Mutex<Vec<Connection>>>,
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
            filter_type: FilterType::None,
            outputs: Arc::new(Mutex::new(Vec::new())),
            envelope: Box::new(adbdr::Filter::new()),
            audio_in: 0.0,
            id,
        }
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
                self.filter_type = input.into();
                info!("setting filter type to {:?}", self.filter_type);
                self.envelope = match self.filter_type {
                    FilterType::None => Box::new(none::Filter::new()),
                    FilterType::ADSR => Box::new(adsr::Filter::new()),
                    FilterType::ADBDR => Box::new(adbdr::Filter::new()),
                    FilterType::OC => Box::new(oc::Filter::new()),
                };
            }
        } else if input_n == AUDIO_IN {
            let audio = samples.iter().sum::<Float>().tanh();
            self.audio_in = audio
        } else if input_n == FILTER_OPEN_IN {
            self.envelope.open_filter(samples.to_vec())
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
