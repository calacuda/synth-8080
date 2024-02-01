use crate::{
    common::{bend_range, notes::Note, Connection, Module},
    osc::{sin_wt::WavetableOscillator, Osc},
    router::{router_read_sync, router_send_sample, Router, RoutingTable},
    Float,
};
use anyhow::{bail, ensure, Result};
use serde::Deserialize;
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};
use tokio::task::{spawn, JoinHandle};
use tracing::{error, info};

pub const N_INPUTS: u8 = 3;
pub const N_OUTPUTS: u8 = 1;

#[derive(Deserialize, Debug, Clone)]
pub enum OscType {
    #[serde(rename = "sine", alias = "sin")]
    Sine,
    #[serde(rename = "square", alias = "squ")]
    Square,
    #[serde(rename = "triangle", alias = "tri")]
    Triangle,
    #[serde(rename = "saw-tooth", alias = "sawtooth", alias = "saw")]
    SawTooth,
}

enum Input {
    Volume = 0,
    PitchBend = 1,
    Pitch = 2,
}

impl TryFrom<u8> for Input {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Volume),
            1 => Ok(Self::PitchBend),
            2 => Ok(Self::Pitch),
            _ => bail!("invalid input number"),
        }
    }
}

pub struct Vco {
    pub routing_table: Router,
    pub osc_type: Arc<Mutex<OscType>>,
    /// the oscilator that produces samples
    pub osc: Arc<Mutex<Box<dyn Osc>>>,
    // /// where the audio samples go
    // pub audio_out: Arc<Mutex<Vec<Connection>>>,
    pub outputs: Arc<Mutex<Vec<Connection>>>,

    // pub playing_out: Arc<Mutex<Vec<Connection>>>,
    /// where the data from the volume input is stored
    pub volume_in: Arc<Mutex<Vec<Float>>>,
    /// where the data from the pitch bend input is stored
    pub pitch_bend_in: Arc<Mutex<Vec<Float>>>,
    /// the note to be played
    pub pitch_in: Arc<Mutex<Vec<Float>>>,
    /// wether the oscilator should produce over tones.
    pub overtones: Arc<Mutex<bool>>,
    /// the thread handle that computes generates the next sample
    pub generator: Arc<Mutex<JoinHandle<()>>>,
    /// the note the oscilator is suposed to make, used to reset self.osc if pitch_bend_in
    /// disconnects
    pub note: Arc<Mutex<Note>>,
    /// how much to bend the pitch when pitch bends happen
    pub bend_amt: Arc<Float>,
}

impl Vco {
    pub fn new(routing_table: Router, id: u8) -> Self {
        let osc_type = Arc::new(Mutex::new(OscType::Sine));
        let osc: Arc<Mutex<Box<dyn Osc>>> =
            Arc::new(Mutex::new(Box::new(WavetableOscillator::new())));
        osc.lock().unwrap().set_frequency(Note::A4.into());
        // let audio_out = Arc::new(Mutex::new(Vec::new()));
        let outputs = Arc::new(Mutex::new(Vec::new()));
        // let playing_out = Arc::new(Mutex::new(Vec::new()));
        let volume_in = Arc::new(Mutex::new(Vec::new()));
        let pitch_bend_in = Arc::new(Mutex::new(Vec::new()));
        let pitch_in = Arc::new(Mutex::new(Vec::new()));
        let overtones = Arc::new(Mutex::new(false));
        let generator = Arc::new(Mutex::new(spawn(async {})));
        let note = Arc::new(Mutex::new(Note::A4));
        let bend_amt = Arc::new(bend_range());

        Self {
            routing_table,
            osc_type,
            osc,
            outputs,
            // playing_out,
            volume_in,
            pitch_bend_in,
            pitch_in,
            overtones,
            generator,
            note,
            bend_amt,
        }
    }

    pub fn connect_to(&self, connection: Connection) -> Result<()> {
        ensure!(
            connection.src_output < N_OUTPUTS,
            "invalid output selection"
        );

        if connection.src_output == 0 {
            ensure!(
                !self.outputs.lock().unwrap().contains(&connection),
                "module already connected"
            );
            self.outputs.lock().unwrap().push(connection);
        } else if connection.src_output == 1 {
            bail!("unhandled valid output selction. in other words a valid output was selected but that output handling code was not yet written.");
        }

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

        if connection.src_output == 0 {
            ensure!(
                self.outputs.lock().unwrap().contains(&connection),
                "module not connected"
            );
            self.outputs
                .lock()
                .unwrap()
                .retain(|out| *out != connection);
        } else if connection.src_output == 1 {
            bail!("unhandled valid output selction. in other words a valid output was selected but that output handling code was not yet written.");
        }

        info!(
            "connected output: {}, of module: {}, to input: {}, of module: {}",
            connection.src_output,
            connection.src_module,
            connection.dest_input,
            connection.dest_module
        );

        Ok(())
    }

    pub fn set_osc_type(&self, osc_type: OscType) {
        let mut ot = self.osc_type.lock().unwrap();
        *ot = osc_type;

        match *ot {
            OscType::Sine => {
                let mut osc = self.osc.lock().unwrap();
                *osc = Box::new(WavetableOscillator::new());
                info!("set to sine wave")
            }
            OscType::Square => error!("not implemented yet"),
            OscType::Triangle => error!("not implemented yet"),
            OscType::SawTooth => error!("not implemented yet"),
        }
    }

    pub fn set_overtones(&self, on: bool) {
        let mut ovt = self.overtones.lock().unwrap();
        *ovt = on;

        // TODO: make twang oscilator for over-tones

        info!("overtones on: {on}")
    }

    /// starts a thread to generate samples.
    pub fn start_event_loop(&self) -> JoinHandle<()> {
        let osc = self.osc.clone();
        let outs = self.outputs.clone();
        let router = self.routing_table.clone();
        // let play_outs = self.play_out.clone();
        // let bend =  self.pitch_bend_in.clone();
        // let bend_amt = self.bend_amt.clone();
        // let note = self.note.clone();

        spawn(async move {
            loop {
                // TODO: get inputs and do the things

                let sample = osc.lock().unwrap().get_sample();

                outs.lock().unwrap().deref().iter().for_each(|con| {
                    if let Err(e) = router_read_sync(router.clone(), *con) {
                        error!("encountered an error waiting for sync message: {e}");
                    }
                });

                outs.lock().unwrap().deref().iter().for_each(|connection| {
                    router_send_sample(router.clone(), *connection, sample);
                });
            }
        })
    }

    /// applies a pitch bend by changing the oscilators frequency
    fn apply_bend(&self, bend: Float, bend_amt: Float) {
        let note = (*self.note.lock().unwrap()).into();
        // let note: Float = (*note).clone().into();

        // get frequency shift
        // self.note + (bend * )
        let shift = bend * (note * bend_amt);

        let new_note = if bend > 0.0 {
            note + (note * shift)
        } else if bend < 0.0 {
            note - (note / shift)
        } else {
            note
        };

        let mut osc = self.osc.lock().unwrap();
        (*osc).set_frequency(new_note);
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

    fn connect(&self, connection: Connection) -> anyhow::Result<()> {
        self.connect_to(connection)?;
        // self.routing_table.inc_connect_counter(connection);
        info!("connecting: {connection:?}");

        Ok(())
    }

    fn disconnect(&self, connection: Connection) -> anyhow::Result<()> {
        // TODO: pop from self.outputs
        self.disconnect_from(connection)?;
        info!("disconnecting: {connection:?}");

        Ok(())
    }
}

// pub async fn start(router: Router) -> anyhow::Result<(Vco, JoinHandle<()>)> {
//     let osc = Vco::new(router);
//     let jh = osc.start()?;
//     Ok((osc, jh))
