use crate::{
    common::{event_loop, Connection, Module},
    router::{ModuleIn, Router},
    Float,
};
use anyhow::Result;
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};
use tokio::{spawn, task::JoinHandle};
use tracing::info;

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
        // info!(
        //     "{} {} = {}",
        //     self.get_env(),
        //     self.get_step(),
        //     self.get_env() + self.get_step()
        // );
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
    /// holds a collection of IO structs which allow for comunication between modules
    pub routing_table: Router,
    /// which filter is currently in use
    pub filter_type: Arc<Mutex<FilterType>>,
    /// where to send the audio that gets generated
    pub outputs: Arc<Mutex<Vec<Connection>>>,
    /// the thread handle that computes generates the next sample
    pub generator: Arc<Mutex<JoinHandle<()>>>,
    /// the filter that is currently in use
    pub envelope: Arc<Mutex<Box<dyn Envelope>>>,
    /// stores the audio input sample
    pub audio_in: Arc<Mutex<Float>>,
    /// the id which identifies this module from all others
    pub id: u8,
}

impl EnvelopeFilter {
    pub fn new(routing_table: Router, id: u8) -> Self {
        Self {
            routing_table,
            filter_type: Arc::new(Mutex::new(FilterType::None)),
            outputs: Arc::new(Mutex::new(Vec::new())),
            generator: Arc::new(Mutex::new(spawn(async {}))),
            envelope: Arc::new(Mutex::new(Box::new(adbdr::Filter::new()))),
            audio_in: Arc::new(Mutex::new(0.0)),
            id,
        }
    }
}

impl Module for EnvelopeFilter {
    fn start(&self) -> anyhow::Result<JoinHandle<()>> {
        let router = self.routing_table.clone();
        let id = self.id as usize;
        // audio output
        let audio = self.audio_in.clone();
        let audio_2 = self.audio_in.clone();

        let outs = self.outputs.clone();
        let env_1 = self.envelope.clone();
        let env_2 = self.envelope.clone();
        let env_3 = self.envelope.clone();
        let env_4 = self.envelope.clone();
        let env_5 = self.envelope.clone();
        let env_6 = self.envelope.clone();
        let env_7 = self.envelope.clone();
        // let env_7 = self.envelope.clone();
        let ft = self.filter_type.clone();

        Ok(spawn(async move {
            let ins: Arc<[ModuleIn]> = (*router)
                .0
                .get(id)
                .expect("this ADBDR Envelope Module was not found in the routing table struct.")
                .clone();
            let gen_sample: Box<dyn FnMut() -> Float + Send> =
                Box::new(move || audio.lock().unwrap().deref() * env_1.lock().unwrap().step());

            let outputs = vec![(outs, gen_sample)];

            let set_filter_type: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let input = samples.iter().sum::<Float>().tanh();

                    if input > 1.0 {
                        let mut ft = ft.lock().unwrap();
                        (*ft) = input.into();
                        info!("setting filter type to {ft:?}");
                        let mut env = env_2.lock().unwrap();
                        *env = match *ft {
                            FilterType::None => Box::new(none::Filter::new()),
                            FilterType::ADSR => Box::new(adsr::Filter::new()),
                            FilterType::ADBDR => Box::new(adbdr::Filter::new()),
                            FilterType::OC => Box::new(oc::Filter::new()),
                        };
                    }
                });

            let set_audio: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let audio = samples.iter().sum::<Float>().tanh();
                    let mut a = audio_2.lock().unwrap();
                    (*a) = audio;
                });

            let open_filter: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    // let sample = samples.iter().sum::<Float>().tanh();
                    env_7.lock().unwrap().open_filter(samples);
                });

            let mod_in_0: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let _ = env_3.lock().unwrap().take_input(0, samples);
                });

            let mod_in_1: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let _ = env_4.lock().unwrap().take_input(1, samples);
                });

            let mod_in_2: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let _ = env_5.lock().unwrap().take_input(2, samples);
                });

            let mod_in_3: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let _ = env_6.lock().unwrap().take_input(3, samples);
                });

            // let mod_in_4: Box<dyn FnMut(Vec<Float>) + Send> =
            //     Box::new(move |samples: Vec<Float>| {
            //         let _ = env_7.lock().unwrap().take_input(4, samples);
            //     });

            let inputs = vec![
                (&ins[FILTER_SELECT_IN as usize], set_filter_type),
                (&ins[AUDIO_IN as usize], set_audio),
                (&ins[FILTER_OPEN_IN as usize], open_filter),
                (&ins[3], mod_in_0),
                (&ins[4], mod_in_1),
                (&ins[5], mod_in_2),
                (&ins[6], mod_in_3),
                // (&ins[7], mod_in_4),
                // (&ins[AUDIO_IN as usize], set_audio),
                // (&ins[AUDIO_IN as usize], set_audio),
                // (&ins[AUDIO_IN as usize], set_audio),
                // (&ins[AUDIO_IN as usize], set_audio),
            ];

            event_loop(router.clone(), inputs, outputs).await;
        }))
    }

    fn n_outputs(&self) -> u8 {
        N_OUTPUTS
    }

    fn connections(&self) -> Arc<Mutex<Vec<Connection>>> {
        self.outputs.clone()
    }
}
