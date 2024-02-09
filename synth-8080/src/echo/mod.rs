use crate::{
    common::{event_loop, Connection, Module},
    router::{ModuleIn, Router},
    Float, SAMPLE_RATE,
};
use anyhow::ensure;
use std::sync::{Arc, Mutex};
use tokio::task::{spawn, JoinHandle};
use tracing::info;

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
    pub routing_table: Router,
    pub buff: Arc<Mutex<Buff>>,
    /// the output where the generated samples are sent
    pub outputs: Arc<Mutex<Vec<Connection>>>,
    /// where the data from the audio input is stored
    pub audio_in: Arc<Mutex<Float>>,
    pub id: u8,
}

impl Echo {
    pub fn new(routing_table: Router, id: u8) -> Self {
        const BUFF_SIZE: usize = SAMPLE_RATE as usize * 5;

        let buff = Arc::new(Mutex::new(Buff {
            size: BUFF_SIZE,
            buff: [0.0; BUFF_SIZE],
            i: 0,
            step: 0,
            volume: 1.0,
        }));
        let outputs = Arc::new(Mutex::new(Vec::new()));
        let audio_in = Arc::new(Mutex::new(0.0));
        // let time_in = Arc::new(Mutex::new(0.0));
        // let decay_in = Arc::new(Mutex::new(0.0));

        Self {
            routing_table,
            buff,
            outputs,
            audio_in,
            // time_in,
            // decay_in,
            id,
        }
    }
}

impl Module for Echo {
    fn start(&self) -> anyhow::Result<JoinHandle<()>> {
        let outs = self.outputs.clone();
        let router = self.routing_table.clone();
        let audio_in = self.audio_in.clone();
        let id = self.id as usize;
        let buff = self.buff.clone();
        let buff_2 = self.buff.clone();
        let buff_3 = self.buff.clone();
        let audio_in_2 = self.audio_in.clone();

        Ok(spawn(async move {
            // prepare call back for event loop
            let ins: Arc<[ModuleIn]> = (*router)
                .0
                .get(id)
                .expect("this Echo module was not found in the routing table struct.")
                .clone();
            let gen_sample: Box<dyn FnMut() -> Float + Send> =
                Box::new(move || buff.lock().unwrap().get_sample(*audio_in.lock().unwrap()));
            let outputs = vec![(outs, gen_sample)];

            let add_audio: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    // let mut b = bend.lock().unwrap();
                    let audio = samples.iter().sum::<Float>() / (samples.len() as Float);
                    let mut a_in_2 = audio_in_2.lock().unwrap();
                    *a_in_2 = audio;
                });
            let speed_set: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    // let mut b = bend.lock().unwrap();
                    let time = samples.iter().sum::<Float>() / (samples.len() as Float);
                    buff_2.lock().unwrap().set_speed(time);
                });
            let set_decay: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    // let mut b = bend.lock().unwrap();
                    let time = samples.iter().sum::<Float>() / (samples.len() as Float);
                    buff_3.lock().unwrap().set_volume(time);
                });

            let inputs = vec![
                (&ins[DECAY_INPUT as usize], set_decay),
                (&ins[AUDIO_INPUT as usize], add_audio),
                (&ins[SPEED_INPUT as usize], speed_set),
            ];

            // start the event loop
            event_loop(router.clone(), inputs, outputs).await;
        }))
    }

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

    fn n_outputs(&self) -> u8 {
        N_OUTPUTS
    }

    fn connections(&self) -> Arc<Mutex<Vec<Connection>>> {
        self.outputs.clone()
    }
}
