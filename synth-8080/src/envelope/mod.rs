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
        // trace!("ADBDR envelope value {}", new_env);
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
    // pub routing_table: Router,
    /// which filter is currently in use
    pub filter_type: FilterType,
    /// where to send the audio that gets generated
    pub outputs: Arc<Mutex<Vec<Connection>>>,
    // /// the thread handle that computes generates the next sample
    // pub generator: Arc<Mutex<JoinHandle>>,
    /// the filter that is currently in use
    pub envelope: Box<dyn Envelope>,
    /// stores the audio input sample
    pub audio_in: Float,
    /// the id which identifies this module from all others
    pub id: u8,
    pub filter_select_in_cons: Arc<Mutex<Vec<Connection>>>,
    pub audio_in_cons: Arc<Mutex<Vec<Connection>>>,
    pub filter_open_cons: Arc<Mutex<Vec<Connection>>>,
    pub in_cons_4: Arc<Mutex<Vec<Connection>>>,
    pub in_cons_5: Arc<Mutex<Vec<Connection>>>,
    pub in_cons_6: Arc<Mutex<Vec<Connection>>>,
    pub in_cons_7: Arc<Mutex<Vec<Connection>>>,
}

impl EnvelopeFilter {
    pub fn new(id: u8) -> Self {
        let filter_select_in_cons = Arc::new(Mutex::new(Vec::new()));
        let audio_in_cons = Arc::new(Mutex::new(Vec::new()));
        let filter_open_cons = Arc::new(Mutex::new(Vec::new()));
        let in_cons_4 = Arc::new(Mutex::new(Vec::new()));
        let in_cons_5 = Arc::new(Mutex::new(Vec::new()));
        let in_cons_6 = Arc::new(Mutex::new(Vec::new()));
        let in_cons_7 = Arc::new(Mutex::new(Vec::new()));

        Self {
            // routing_table,
            filter_type: FilterType::None,
            outputs: Arc::new(Mutex::new(Vec::new())),
            // generator: Arc::new(Mutex::new(spawn(async {}))),
            envelope: Box::new(adbdr::Filter::new()),
            audio_in: 0.0,
            id,
            filter_select_in_cons,
            audio_in_cons,
            filter_open_cons,
            in_cons_4,
            in_cons_5,
            in_cons_6,
            in_cons_7,
        }
    }
}

impl Module for EnvelopeFilter {
    async fn get_samples(&mut self) -> Vec<(u8, Float)> {
        vec![(0, self.audio_in * self.envelope.step())]
    }

    async fn recv_samples(&mut self, input_n: u8, samples: &[Float]) {
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
            self.envelope.take_input(0, samples.to_vec());
        } else if input_n == 4 {
            self.envelope.take_input(1, samples.to_vec());
        } else if input_n == 5 {
            self.envelope.take_input(2, samples.to_vec());
        } else if input_n == 6 {
            self.envelope.take_input(3, samples.to_vec());
        } else {
            error!("invalid input selection {:?}:{input_n}", self.filter_type);
        }
    }

    // fn start(&self) -> anyhow::Result<JoinHandle> {
    //     let router = self.routing_table.clone();
    //     let id = self.id as usize;
    //     // audio output
    //     let audio = self.audio_in.clone();
    //     let audio_2 = self.audio_in.clone();
    //
    //     let outs = self.outputs.clone();
    //     let env_1 = self.envelope.clone();
    //     let env_2 = self.envelope.clone();
    //     let env_3 = self.envelope.clone();
    //     let env_4 = self.envelope.clone();
    //     let env_5 = self.envelope.clone();
    //     let env_6 = self.envelope.clone();
    //     let env_7 = self.envelope.clone();
    //     // let env_7 = self.envelope.clone();
    //     let ft = self.filter_type.clone();
    //
    //     let fs_in_cons = self.filter_select_in_cons.clone();
    //     let audio_in_cons = self.audio_in_cons.clone();
    //     let filter_open_cons = self.filter_open_cons.clone();
    //     let in_mod_in_3 = self.in_cons_4.clone();
    //     let in_mod_in_4 = self.in_cons_5.clone();
    //     let in_mod_in_5 = self.in_cons_6.clone();
    //     let in_mod_in_6 = self.in_cons_7.clone();
    //
    //     Ok(spawn(async move {
    //         // let ins: Arc<[ModuleIn]> = (*router)
    //         //     .in_s
    //         //     .get(id)
    //         //     .expect("this ADBDR Envelope Module was not found in the routing table struct.")
    //         //     .clone();
    //         let gen_sample: Box<dyn FnMut() -> Float + Send> =
    //             Box::new(move || audio.lock().unwrap().deref() * env_1.lock().unwrap().step());
    //
    //         let outputs = (id, vec![(outs, gen_sample)]);
    //
    //         let set_filter_type: Box<dyn FnMut(Vec<Float>) + Send> =
    //             Box::new(move |samples: Vec<Float>| {
    //                 let input = samples.iter().sum::<Float>().tanh();
    //
    //                 if input > 1.0 {
    //                     let mut ft = ft.lock().unwrap();
    //                     (*ft) = input.into();
    //                     info!("setting filter type to {ft:?}");
    //                     let mut env = env_2.lock().unwrap();
    //                     *env = match *ft {
    //                         FilterType::None => Box::new(none::Filter::new()),
    //                         FilterType::ADSR => Box::new(adsr::Filter::new()),
    //                         FilterType::ADBDR => Box::new(adbdr::Filter::new()),
    //                         FilterType::OC => Box::new(oc::Filter::new()),
    //                     };
    //                 }
    //             });
    //
    //         let set_audio: Box<dyn FnMut(Vec<Float>) + Send> =
    //             Box::new(move |samples: Vec<Float>| {
    //                 let audio = samples.iter().sum::<Float>().tanh();
    //                 let mut a = audio_2.lock().unwrap();
    //                 (*a) = audio;
    //             });
    //
    //         let open_filter: Box<dyn FnMut(Vec<Float>) + Send> =
    //             Box::new(move |samples: Vec<Float>| {
    //                 // let sample = samples.iter().sum::<Float>().tanh();
    //                 // info!("open_filter");
    //                 env_7.lock().unwrap().open_filter(samples);
    //             });
    //
    //         let mod_in_0: Box<dyn FnMut(Vec<Float>) + Send> =
    //             Box::new(move |samples: Vec<Float>| {
    //                 let _ = env_3.lock().unwrap().take_input(0, samples);
    //             });
    //
    //         let mod_in_1: Box<dyn FnMut(Vec<Float>) + Send> =
    //             Box::new(move |samples: Vec<Float>| {
    //                 let _ = env_4.lock().unwrap().take_input(1, samples);
    //             });
    //
    //         let mod_in_2: Box<dyn FnMut(Vec<Float>) + Send> =
    //             Box::new(move |samples: Vec<Float>| {
    //                 let _ = env_5.lock().unwrap().take_input(2, samples);
    //             });
    //
    //         let mod_in_3: Box<dyn FnMut(Vec<Float>) + Send> =
    //             Box::new(move |samples: Vec<Float>| {
    //                 let _ = env_6.lock().unwrap().take_input(3, samples);
    //             });
    //
    //         // let mod_in_4: Box<dyn FnMut(Vec<Float>) + Send> =
    //         //     Box::new(move |samples: Vec<Float>| {
    //         //         let _ = env_7.lock().unwrap().take_input(4, samples);
    //         //     });
    //
    //         let inputs = vec![
    //             (fs_in_cons, set_filter_type),
    //             (audio_in_cons, set_audio),
    //             (filter_open_cons, open_filter),
    //             (in_mod_in_3, mod_in_0),
    //             (in_mod_in_4, mod_in_1),
    //             (in_mod_in_5, mod_in_2),
    //             (in_mod_in_6, mod_in_3),
    //             // (&ins[7], mod_in_4),
    //             // (&ins[AUDIO_IN as usize], set_audio),
    //             // (&ins[AUDIO_IN as usize], set_audio),
    //             // (&ins[AUDIO_IN as usize], set_audio),
    //             // (&ins[AUDIO_IN as usize], set_audio),
    //         ];
    //
    //         event_loop(router.clone(), inputs, outputs).await;
    //     }))
    // }
    //
    // fn connect(&self, connection: Connection) -> anyhow::Result<()> {
    //     // self.connect_auido_out_to(connection)?;
    //     // self.routing_table.inc_connect_counter(connection);
    //     // info!("connecting: {connection:?}");
    //     if connection.dest_input == FILTER_SELECT_IN {
    //         self.filter_select_in_cons.lock().unwrap().push(connection);
    //     } else if connection.dest_input == AUDIO_IN {
    //         self.audio_in_cons.lock().unwrap().push(connection);
    //     } else if connection.dest_input == FILTER_OPEN_IN {
    //         self.filter_open_cons.lock().unwrap().push(connection);
    //     } else if connection.dest_input == 3 {
    //         self.in_cons_4.lock().unwrap().push(connection);
    //     } else if connection.dest_input == 4 {
    //         self.in_cons_5.lock().unwrap().push(connection);
    //     } else if connection.dest_input == 5 {
    //         self.in_cons_6.lock().unwrap().push(connection);
    //     } else if connection.dest_input == 6 {
    //         self.in_cons_7.lock().unwrap().push(connection);
    //     } else {
    //         bail!("invalid input selection");
    //     }
    //
    //     Ok(())
    // }
    //
    // fn disconnect(&self, connection: Connection) -> anyhow::Result<()> {
    //     // self.disconnect_from(connection)?;
    //     // info!("disconnecting: {connection:?}");
    //     if connection.dest_input == FILTER_SELECT_IN {
    //         self.filter_select_in_cons
    //             .lock()
    //             .unwrap()
    //             .retain(|con| *con != connection);
    //     } else if connection.dest_input == AUDIO_IN {
    //         self.audio_in_cons
    //             .lock()
    //             .unwrap()
    //             .retain(|con| *con != connection);
    //     } else if connection.dest_input == FILTER_OPEN_IN {
    //         self.filter_open_cons
    //             .lock()
    //             .unwrap()
    //             .retain(|con| *con != connection);
    //     } else if connection.dest_input == 3 {
    //         self.in_cons_4
    //             .lock()
    //             .unwrap()
    //             .retain(|con| *con != connection);
    //     } else if connection.dest_input == 4 {
    //         self.in_cons_5
    //             .lock()
    //             .unwrap()
    //             .retain(|con| *con != connection);
    //     } else if connection.dest_input == 5 {
    //         self.in_cons_6
    //             .lock()
    //             .unwrap()
    //             .retain(|con| *con != connection);
    //     } else if connection.dest_input == 6 {
    //         self.in_cons_7
    //             .lock()
    //             .unwrap()
    //             .retain(|con| *con != connection);
    //     } else {
    //         bail!("invalid input selection");
    //     }
    //
    //     Ok(())
    // }

    // fn n_outputs(&self) -> u8 {
    //     N_OUTPUTS
    // }
    //
    // fn connections(&self) -> Arc<Mutex<Vec<Connection>>> {
    //     self.outputs.clone()
    // }
}
