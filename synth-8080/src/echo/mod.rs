use crate::{
    common::{Connection, Module},
    Float, SAMPLE_RATE,
};
use std::sync::{Arc, Mutex};
use tracing::*;

pub const N_INPUTS: u8 = 3;
pub const N_OUTPUTS: u8 = 1;

pub const AUDIO_INPUT: u8 = 0;
pub const SPEED_INPUT: u8 = 1;
pub const DECAY_INPUT: u8 = 2;

pub struct Buff {
    pub size: usize,
    pub buff: [Float; SAMPLE_RATE as usize * 5],
    pub i: usize,
    pub step: usize,
    pub volume: Float,
}

impl Buff {
    pub fn get_sample(&mut self, input_sample: Float) -> Float {
        let echo = ((self.buff[self.i] * self.volume) + input_sample) * 0.5;
        self.buff[self.i] = echo;
        self.i = (self.i + 1 + self.step) % self.size;
        echo
    }

    /// sets speed, takes speed in seconds
    pub fn set_speed(&mut self, speed: Float) {
        self.step = (SAMPLE_RATE as Float * speed) as usize;
    }

    pub fn set_volume(&mut self, volume: Float) {
        self.volume = volume;
    }
}

pub struct Echo {
    // pub routing_table: Router,
    buff: Buff,
    /// the output where the generated samples are sent
    outputs: Arc<Mutex<Vec<Connection>>>,
    /// where the data from the audio input is stored
    audio_in: Float,
    id: u8,
    audio_cons: Arc<Mutex<Vec<Connection>>>,
    speed_cons: Arc<Mutex<Vec<Connection>>>,
    decay_cons: Arc<Mutex<Vec<Connection>>>,
}

impl Echo {
    pub fn new(id: u8) -> Self {
        const BUFF_SIZE: usize = SAMPLE_RATE as usize * 5;

        let buff = Buff {
            size: BUFF_SIZE,
            buff: [0.0; BUFF_SIZE],
            i: 0,
            step: 0,
            volume: 1.0,
        };
        let outputs = Arc::new(Mutex::new(Vec::new()));
        let audio_in = 0.0;
        // let time_in = Arc::new(Mutex::new(0.0));
        // let decay_in = Arc::new(Mutex::new(0.0));
        let audio_cons = Arc::new(Mutex::new(Vec::new()));
        let speed_cons = Arc::new(Mutex::new(Vec::new()));
        let decay_cons = Arc::new(Mutex::new(Vec::new()));

        Self {
            // routing_table,
            buff,
            outputs,
            audio_in,
            // time_in,
            // decay_in,
            id,
            audio_cons,
            speed_cons,
            decay_cons,
        }
    }
}

impl Module for Echo {
    async fn get_samples(&mut self) -> Vec<(u8, Float)> {
        vec![(0, self.buff.get_sample(self.audio_in))]
    }

    async fn recv_samples(&mut self, input_n: u8, samples: &[Float]) {
        let sample: Float = samples.iter().sum();

        if input_n == AUDIO_INPUT {
            self.audio_in = sample.tanh();
        } else if input_n == SPEED_INPUT {
            self.buff.set_speed(sample.tanh());
        } else if input_n == DECAY_INPUT {
            self.buff.set_volume(sample.tanh());
        } else {
            error!("invalid input for echo module: {input_n}");
        }
    }

    // fn start(&self) -> anyhow::Result<JoinHandle> {
    //     let outs = self.outputs.clone();
    //     let router = self.routing_table.clone();
    //     let audio_in = self.audio_in.clone();
    //     let id = self.id as usize;
    //     let buff = self.buff.clone();
    //     let buff_2 = self.buff.clone();
    //     let buff_3 = self.buff.clone();
    //     let audio_in_2 = self.audio_in.clone();
    //
    //     let audio_cons = self.audio_cons.clone();
    //     let decay_cons = self.decay_cons.clone();
    //     let speed_cons = self.speed_cons.clone();
    //
    //     Ok(spawn(async move {
    //         // prepare call back for event loop
    //         // let ins: Arc<[ModuleIn]> = (*router)
    //         //     .in_s
    //         //     .get(id)
    //         //     .expect("this Echo module was not found in the routing table struct.")
    //         //     .0
    //         //     .clone();
    //         let gen_sample: Box<dyn FnMut() -> Float + Send> =
    //             Box::new(move || buff.lock().unwrap().get_sample(*audio_in.lock().unwrap()));
    //         let outputs = (id, vec![(outs, gen_sample)]);
    //
    //         let add_audio: Box<dyn FnMut(Vec<Float>) + Send> =
    //             Box::new(move |samples: Vec<Float>| {
    //                 // let mut b = bend.lock().unwrap();
    //                 let audio = samples.iter().sum::<Float>() / (samples.len() as Float);
    //                 let mut a_in_2 = audio_in_2.lock().unwrap();
    //                 *a_in_2 = audio;
    //             });
    //         let speed_set: Box<dyn FnMut(Vec<Float>) + Send> =
    //             Box::new(move |samples: Vec<Float>| {
    //                 // let mut b = bend.lock().unwrap();
    //                 let time = samples.iter().sum::<Float>() / (samples.len() as Float);
    //                 buff_2.lock().unwrap().set_speed(time);
    //             });
    //         let set_decay: Box<dyn FnMut(Vec<Float>) + Send> =
    //             Box::new(move |samples: Vec<Float>| {
    //                 // let mut b = bend.lock().unwrap();
    //                 let time = samples.iter().sum::<Float>() / (samples.len() as Float);
    //                 buff_3.lock().unwrap().set_volume(time);
    //             });
    //
    //         let inputs = vec![
    //             (decay_cons, set_decay),
    //             (audio_cons, add_audio),
    //             (speed_cons, speed_set),
    //         ];
    //
    //         // start the event loop
    //         event_loop(router.clone(), inputs, outputs).await;
    //     }))
    // }

    // fn connect(&self, connection: Connection) -> anyhow::Result<()> {
    //     ensure!(
    //         connection.src_output < N_OUTPUTS,
    //         "invalid output selection"
    //     );
    //     ensure!(
    //         !self.outputs.lock().unwrap().contains(&connection),
    //         "module already connected"
    //     );
    //
    //     self.outputs.lock().unwrap().push(connection);
    //
    //     info!(
    //         "connected output: {}, of module: {}, to input: {}, of module: {}",
    //         connection.src_output,
    //         connection.src_module,
    //         connection.dest_input,
    //         connection.dest_module
    //     );
    //
    //     Ok(())
    // }
    //
    // fn disconnect(&self, connection: Connection) -> anyhow::Result<()> {
    //     ensure!(
    //         connection.src_output < N_OUTPUTS,
    //         "invalid output selection"
    //     );
    //     ensure!(
    //         self.outputs.lock().unwrap().contains(&connection),
    //         "module not connected"
    //     );
    //
    //     self.outputs
    //         .lock()
    //         .unwrap()
    //         .retain(|out| *out != connection);
    //
    //     info!(
    //         "connected output: {}, of module: {}, to input: {}, of module: {}",
    //         connection.src_output,
    //         connection.src_module,
    //         connection.dest_input,
    //         connection.dest_module
    //     );
    //
    //     Ok(())
    // }

    // fn n_outputs(&self) -> u8 {
    //     N_OUTPUTS
    // }

    // fn connections(&self) -> Arc<Mutex<Vec<Connection>>> {
    //     self.outputs.clone()
    // }

    // fn connect(&self, connection: Connection) -> anyhow::Result<()> {
    //     // self.connect_auido_out_to(connection)?;
    //     // // self.routing_table.inc_connect_counter(connection);
    //     // info!("connecting: {connection:?}");
    //     if connection.dest_input == AUDIO_INPUT {
    //         self.audio_cons.lock().unwrap().push(connection);
    //     } else if connection.dest_input == SPEED_INPUT {
    //         self.speed_cons.lock().unwrap().push(connection);
    //     } else if connection.dest_input == DECAY_INPUT {
    //         self.decay_cons.lock().unwrap().push(connection);
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
    //     if connection.dest_input == AUDIO_INPUT {
    //         self.audio_cons
    //             .lock()
    //             .unwrap()
    //             .retain(|con| *con != connection);
    //     } else if connection.dest_input == SPEED_INPUT {
    //         self.speed_cons
    //             .lock()
    //             .unwrap()
    //             .retain(|con| *con != connection);
    //     } else if connection.dest_input == DECAY_INPUT {
    //         self.decay_cons
    //             .lock()
    //             .unwrap()
    //             .retain(|con| *con != connection);
    //     } else {
    //         bail!("invalid input selection");
    //     }
    //
    //     Ok(())
    // }
}
