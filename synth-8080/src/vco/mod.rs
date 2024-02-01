use crate::{
    common::{bend_range, notes::Note, Connection},
    osc::{sin_wt::WavetableOscillator, Osc},
    router::{get_idx, router_set, Router},
    Float,
};
use anyhow::{bail, ensure, Result};
use serde::Deserialize;
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};
use tokio::{
    task::{spawn, JoinHandle},
    // time::{sleep, Duration},
};
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
    /// where the audio samples go
    pub audio_out: Arc<Mutex<Vec<Connection>>>,

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
    pub fn new(routing_table: Router) -> Self {
        let osc_type = Arc::new(Mutex::new(OscType::Sine));
        let osc: Arc<Mutex<Box<dyn Osc>>> =
            Arc::new(Mutex::new(Box::new(WavetableOscillator::new())));
        let audio_out = Arc::new(Mutex::new(Vec::new()));
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
            audio_out,
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

    pub async fn connect_to(
        &self,
        output_select: u8,
        dest_module: u8,
        input_select: u8,
    ) -> Result<()> {
        ensure!(output_select < N_OUTPUTS, "invalid output selection");

        let connection = Connection {
            idx: get_idx(self.routing_table.clone(), dest_module, input_select).await,
            src_output: output_select,
            dest_module,
            dest_input: input_select,
        };

        if output_select == 0 {
            self.audio_out.lock().unwrap().push(connection);
        } else {
            bail!("unhandled valid output selction. in other words a valid output was selected but that output handling code waas not yet written.");
        }

        Ok(())
    }

    pub fn set_osc_type(&self, osc_type: OscType) -> String {
        let mut ot = self.osc_type.lock().unwrap();
        *ot = osc_type;

        match *ot {
            OscType::Sine => {
                let mut osc = self.osc.lock().unwrap();
                *osc = Box::new(WavetableOscillator::new());
                "set to sine wave".to_string()
            }
            OscType::Square => "not implemented yet".to_string(),
            OscType::Triangle => "not implemented yet".to_string(),
            OscType::SawTooth => "not implemented yet".to_string(),
        }
    }

    pub fn set_overtones(&self, on: bool) -> String {
        let mut ovt = self.overtones.lock().unwrap();
        *ovt = on;

        // TODO: make twang oscilator for over-tones

        format!("overtones on: {on}")
    }

    /// starts a thread to generate samples.
    pub fn start(&self) -> JoinHandle<()> {
        let osc = self.osc.clone();
        let outs = self.audio_out.clone();
        let router = self.routing_table.clone();
        // let play_outs = self.play_out.clone();
        // let bend =  self.pitch_bend_in.clone();
        // let bend_amt = self.bend_amt.clone();
        // let note = self.note.clone();

        spawn(async move {
            // let mut last_send = Instant::now();
            // let dur = Duration::from_nanos(20833);
            // let dur = Duration::from_nanos(0);

            loop {
                // sleep(dur).await;
                // let bends = bend.lock().unwrap();
                // self.apply_bend(bends.iter().sum::<Float>() / bends.len() as Float);
                // osc.lock().unwrap().set_frequency(Vco::apply_bend(
                //     note.lock().unwrap().clone().into(),
                //     bends.iter().sum::<Float>() / bends.len() as Float,
                //     *bend_amt,
                // ));
                let con = outs.lock().unwrap()[0];

                if router[con]
                    .lock()
                    .unwrap()
                    .get(con.idx)
                    .map_or(None, |f| *f)
                    .is_none()
                {
                    let sample = osc.lock().unwrap().get_sample();
                    // info!("{sample:?}");

                    // while router[con]
                    //     .lock()
                    //     .unwrap()
                    //     .get(con.idx)
                    //     .map_or(None, |f| *f)
                    //     .is_some()
                    // {
                    //     // error!("sleeping");
                    //     // sleep(dur);
                    // }

                    // error!("sample -> {sample}");
                    outs.lock().unwrap().deref().iter().for_each(|connection| {
                        router_set(router.clone(), *connection, sample);
                    });
                }
            }
        })
    }

    /// applies a pitch bend by changing the oscilators frequency
    fn apply_bend(note: Float, bend: Float, bend_amt: Float) -> Float {
        // let note = self.note.lock().unwrap();
        // let note: Float = (*note).clone().into();

        // get frequency shift
        // self.note + (bend * )
        let shift = bend * (note * bend_amt);

        // let mut osc = self.osc.lock().unwrap();
        if bend > 0.0 {
            note + (note * shift)
        } else if bend < 0.0 {
            note - (note / shift)
        } else {
            note
        }
    }

    pub fn set_note(&self, note: Note) -> String {
        let mut n = self.note.lock().unwrap();
        // get frequency from note
        *n = note;
        self.osc.lock().unwrap().set_frequency(n.clone().into());

        format!("set note to {n}")
    }
}

pub async fn start(router: Router) -> anyhow::Result<(Vco, JoinHandle<()>)> {
    let osc = Vco::new(router);
    let jh = osc.start();
    Ok((osc, jh))
}
