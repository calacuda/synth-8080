use crate::{
    common::{bend_range, event_loop, notes::Note, Connection, Module},
    osc::{OscType, Oscilator},
    router::{ModuleIn, Router},
    Float,
};
use anyhow::{ensure, Result};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};
use tokio::task::{spawn, JoinHandle};
use tracing::info;

pub const N_INPUTS: u8 = 3;
pub const N_OUTPUTS: u8 = 1;
pub const VOLUME_INPUT: u8 = 0;
pub const PITCH_INPUT: u8 = 1;
pub const PITCH_BEND_INPUT: u8 = 2;

pub struct Vco {
    /// holds a collection of IO structs which allow for comunication between modules
    pub routing_table: Router,
    /// stores the current oscilator type (probably not nessesary)
    pub osc_type: Arc<Mutex<OscType>>,
    /// the oscilator that produces samples
    pub osc: Arc<Mutex<Oscilator>>,
    /// where to send the audio that gets generated
    pub outputs: Arc<Mutex<Vec<Connection>>>,
    /// where the data from the volume input is stored
    pub volume_in: Arc<Mutex<Float>>,
    /// where the data from the pitch bend input is stored
    pub pitch_bend_in: Arc<Mutex<Float>>,
    /// the note to be played
    pub pitch_in: Arc<Mutex<Float>>,
    /// wether the oscilator should produce over tones.
    pub overtones: Arc<Mutex<bool>>,
    /// the thread handle that computes generates the next sample
    pub generator: Arc<Mutex<JoinHandle<()>>>,
    /// the note the oscilator is suposed to make, used to reset self.osc if pitch_bend_in
    /// disconnects
    pub note: Arc<Mutex<Note>>,
    /// how much to bend the pitch when pitch bends happen
    pub bend_amt: Arc<Float>,
    /// the id of this module, must corespond to its index in the routing table
    pub id: u8,
}

impl Vco {
    pub fn new(routing_table: Router, id: u8) -> Self {
        let osc_type = Arc::new(Mutex::new(OscType::Sine));
        // TODO: test wavetable
        let osc = Arc::new(Mutex::new(Oscilator::new()));
        let outputs = Arc::new(Mutex::new(Vec::new()));
        let volume_in = Arc::new(Mutex::new(1.0));
        let pitch_bend_in = Arc::new(Mutex::new(0.0));
        let pitch_in = Arc::new(Mutex::new(0.0));
        let overtones = Arc::new(Mutex::new(false));
        let generator = Arc::new(Mutex::new(spawn(async {})));
        let note = Arc::new(Mutex::new(Note::A4));
        let bend_amt = Arc::new(bend_range());

        // DEBUG
        // osc.lock().unwrap().set_frequency(Note::A4.into());
        // osc.lock().unwrap().set_overtones(true);
        // osc.lock().unwrap().set_waveform(OscType::Triangle);

        Self {
            routing_table,
            osc_type,
            osc,
            outputs,
            volume_in,
            pitch_bend_in,
            pitch_in,
            overtones,
            generator,
            note,
            bend_amt,
            id,
        }
    }

    pub fn connect_auido_out_to(&self, connection: Connection) -> Result<()> {
        ensure!(
            connection.src_output < N_OUTPUTS,
            "invalid output selection"
        );
        ensure!(
            !self.outputs.lock().unwrap().contains(&connection),
            "module already connected"
        );
        self.outputs.lock().unwrap().push(connection);

        Ok(())
    }

    pub fn disconnect_from(&self, connection: Connection) -> Result<()> {
        ensure!(
            connection.src_output < N_OUTPUTS,
            "invalid output selection"
        );
        ensure!(
            self.outputs.lock().unwrap().contains(&connection),
            "module not connected"
        );
        self.outputs
            .lock()
            .unwrap()
            .retain(|out| *out != connection);
        // info!(
        //     "connected output: {}, of module: {}, to input: {}, of module: {}",
        //     connection.src_output,
        //     connection.src_module,
        //     connection.dest_input,
        //     connection.dest_module
        // );

        Ok(())
    }

    pub fn set_osc_type(&self, osc_type: OscType) {
        let mut ot = self.osc_type.lock().unwrap();

        if osc_type != *ot {
            *ot = osc_type;
            self.osc.lock().unwrap().set_waveform(*ot);
            info!("set to {osc_type:?}");
        }
    }

    pub fn set_overtones(&self, on: bool) {
        let mut ovt = self.overtones.lock().unwrap();
        *ovt = on;

        info!("overtones on: {on}")
    }

    /// starts a thread to generate samples.
    pub fn start_event_loop(&self) -> JoinHandle<()> {
        let osc = self.osc.clone();
        let outs = self.outputs.clone();
        let router = self.routing_table.clone();
        let volume = self.volume_in.clone();
        let volume_2 = self.volume_in.clone();
        let id = self.id as usize;
        let pitch = self.pitch_in.clone();
        let osc_2 = self.osc.clone();
        let osc_3 = self.osc.clone();

        spawn(async move {
            // prepare call back for event loop
            let ins: Arc<[ModuleIn]> = (*router)
                .0
                .get(id)
                .expect("this VCO Module was not found in the routing table struct.")
                .clone();
            let gen_sample: Box<dyn FnMut() -> Float + Send> = Box::new(move || {
                let sample = osc.lock().unwrap().get_sample();
                // info!("volume is {}", volume_2.lock().unwrap());
                sample * volume_2.lock().unwrap().deref()
            });
            let outputs = vec![(outs, gen_sample)];
            let update_volume: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let mut v = volume.lock().unwrap();
                    let tmp_v = samples.iter().sum::<Float>().tanh();
                    *v = (tmp_v * 0.5) + 0.5;
                });
            let set_pitch: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    // info!("got pitches");
                    let mut p = pitch.lock().unwrap();
                    *p = samples[samples.len() - 1];
                    let mut osc = osc_3.lock().unwrap();
                    (*osc).set_frequency(*p);
                    // info!("setting pitch to {p}");
                });
            let bend_pitch: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    // let mut b = bend.lock().unwrap();
                    let bend = samples.iter().sum::<Float>().tanh();
                    osc_2.lock().unwrap().apply_bend(bend);
                });
            let inputs = vec![
                (&ins[VOLUME_INPUT as usize], update_volume),
                (&ins[PITCH_INPUT as usize], set_pitch),
                (&ins[PITCH_BEND_INPUT as usize], bend_pitch),
            ];

            // start the event loop
            event_loop(router.clone(), inputs, outputs).await;
        })
    }

    pub fn set_note(&self, note: Note) {
        let mut n = self.note.lock().unwrap();
        // get frequency from note
        *n = note;
        self.osc.lock().unwrap().set_frequency(n.clone().into());

        info!("set note to {n}")
    }
}

impl Module for Vco {
    fn start(&self) -> anyhow::Result<JoinHandle<()>> {
        Ok(self.start_event_loop())
    }

    // fn connect(&self, connection: Connection) -> anyhow::Result<()> {
    //     self.connect_auido_out_to(connection)?;
    //     // self.routing_table.inc_connect_counter(connection);
    //     // info!("connecting: {connection:?}");
    //
    //     Ok(())
    // }
    //
    // fn disconnect(&self, connection: Connection) -> anyhow::Result<()> {
    //     self.disconnect_from(connection)?;
    //     // info!("disconnecting: {connection:?}");
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
