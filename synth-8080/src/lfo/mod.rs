use crate::{
    common::{event_loop, Connection, Module},
    osc::{OscType, Oscilator},
    router::{ModuleIn, Router},
    Float,
};
use anyhow::{ensure, Result};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};
use tokio::task::{spawn, JoinHandle};
use tracing::info;

pub const N_INPUTS: u8 = 3;
pub const N_OUTPUTS: u8 = 1;
pub const PITCH_IN: u8 = 0;
pub const VOL_IN: u8 = 1;
pub const OSC_TYPE_IN: u8 = 2;

pub struct Lfo {
    pub routing_table: Router,
    pub osc_type: Arc<Mutex<OscType>>,
    /// the oscilator that produces samples
    pub osc: Arc<Mutex<Oscilator>>,
    /// the output where the generated samples are sent
    pub outputs: Arc<Mutex<Vec<Connection>>>,
    /// where the data from the volume input is stored
    pub volume_in: Arc<Mutex<Float>>,
    /// the note to be played
    pub pitch_in: Arc<Mutex<Float>>,
    pub id: u8,
}

impl Lfo {
    pub fn new(routing_table: Router, id: u8) -> Self {
        let osc_type = Arc::new(Mutex::new(OscType::Sine));
        let osc = Arc::new(Mutex::new(Oscilator::new()));
        let outputs = Arc::new(Mutex::new(Vec::new()));
        let volume_in = Arc::new(Mutex::new(1.0));
        let pitch_in = Arc::new(Mutex::new(5.0));

        // DEBUG
        osc.lock().unwrap().set_frequency(2.5);
        (*volume_in.lock().unwrap().deref_mut()) = 0.25;

        Self {
            routing_table,
            osc_type,
            osc,
            outputs,
            volume_in,
            pitch_in,
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

        info!(
            "connected output: {}, of module: {}, to input: {}, of module: {}",
            connection.src_output,
            connection.src_module,
            connection.dest_input,
            connection.dest_module
        );

        Ok(())
    }

    pub fn disconnect_from(&self, connection: Connection) -> Result<()> {
        ensure!(
            connection.src_output < N_OUTPUTS,
            "invalid output selection"
        );

        // if connection.src_output == VOL_IN {
        //     ensure!(
        //         self.outputs.lock().unwrap().contains(&connection),
        //         "module not connected"
        //     );
        //     self.outputs
        //         .lock()
        //         .unwrap()
        //         .retain(|out| *out != connection);
        // } else if connection.src_output == PITCH_IN {
        //     bail!("unhandled valid output selction. in other words a valid output was selected but that output handling code was not yet written.");
        // }
        ensure!(
            self.outputs.lock().unwrap().contains(&connection),
            "module not connected"
        );
        // info!("outputs => {:?}", self.outputs.lock().unwrap());
        self.outputs
            .lock()
            .unwrap()
            .retain(|&out| out != connection);
        // info!("outputs => {:?}", self.outputs.lock().unwrap());

        // info!(
        //     "disconnected output: {}, of module: {}, to input: {}, of module: {}",
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

    pub fn set_pitch(&self, pitch: Float) {
        self.osc.lock().unwrap().set_frequency(pitch);
    }

    /// starts a thread to generate samples.
    pub fn start_event_loop(&self) -> JoinHandle<()> {
        // let empty_vec = Vec::new();
        let osc = self.osc.clone();
        let audio_outs = self.outputs.clone();
        let router = self.routing_table.clone();
        let volume = self.volume_in.clone();
        let volume_2 = self.volume_in.clone();
        let pitch = self.pitch_in.clone();
        let id = self.id as usize;

        spawn(async move {
            // prepare call back for event loop
            let ins: &Vec<ModuleIn> = (*router)
                .0
                .get(id)
                .expect("this LFO Module was not found in the routing table struct.")
                .as_ref();
            let gen_sample: Box<dyn FnMut() -> Float + Send> = Box::new(move || {
                let sample = osc.lock().unwrap().get_sample() * volume_2.lock().unwrap().deref();
                // info!("lfo output volume {}", sample);
                sample
            });
            let update_volume: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let mut v = volume.lock().unwrap();
                    *v = samples.iter().sum::<Float>() / (samples.len() as Float);
                });
            let outputs = vec![(audio_outs, gen_sample)];
            let update_pitch: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let mut p = pitch.lock().unwrap();
                    *p = (samples.iter().sum::<Float>() / (samples.len() as Float)).abs() * 220.0;
                });

            let inputs = vec![
                (&ins[VOL_IN as usize], update_volume),
                (&ins[PITCH_IN as usize], update_pitch),
            ];

            // start the event loop
            event_loop(router.clone(), inputs, outputs).await;
        })
    }
}

impl Module for Lfo {
    fn start(&self) -> anyhow::Result<JoinHandle<()>> {
        Ok(self.start_event_loop())
    }

    fn connect(&self, connection: Connection) -> anyhow::Result<()> {
        self.connect_auido_out_to(connection)?;
        // self.routing_table.inc_connect_counter(connection);
        info!("connecting: {connection:?}");

        Ok(())
    }

    fn disconnect(&self, connection: Connection) -> anyhow::Result<()> {
        self.disconnect_from(connection)?;
        info!("disconnecting: {connection:?}");

        Ok(())
    }
}
