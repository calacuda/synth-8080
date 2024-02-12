use crate::{
    common::{bend_range, notes::Note, Module},
    osc::{OscType, Oscilator},
    Float,
};
use std::sync::Arc;
use tracing::*;

pub const N_INPUTS: u8 = 3;
pub const N_OUTPUTS: u8 = 1;
pub const VOLUME_INPUT: u8 = 0;
pub const PITCH_INPUT: u8 = 1;
pub const PITCH_BEND_INPUT: u8 = 2;

pub struct Vco {
    /// stores the current oscilator type (probably not nessesary)
    pub osc_type: OscType,
    /// the oscilator that produces samples
    pub osc: Oscilator,
    /// where the data from the volume input is stored
    pub volume_in: Float,
    /// the note to be played
    pub pitch_in: Float,
    /// wether the oscilator should produce over tones.
    pub overtones: bool,
    pub note: Note,
    /// how much to bend the pitch when pitch bends happen
    pub bend_amt: Arc<Float>,
    /// the id of this module, must corespond to its index in the routing table
    pub id: u8,
}

impl Vco {
    pub fn new(id: u8) -> Self {
        let osc_type = OscType::Sine;
        let osc = Oscilator::new();
        let volume_in = 1.0;
        let pitch_in = 0.0;
        let overtones = false;
        let note = Note::A4;
        let bend_amt = Arc::new(bend_range());

        // DEBUG
        // osc.set_frequency(Note::A4.into());
        // osc.set_overtones(true);
        // osc.lock().unwrap().set_waveform(OscType::Triangle);

        Self {
            osc_type,
            osc,
            volume_in,
            pitch_in,
            overtones,
            note,
            bend_amt,
            id,
        }
    }

    pub fn set_osc_type(&mut self, osc_type: OscType) {
        if osc_type != self.osc_type {
            self.osc_type = osc_type;
            self.osc.set_waveform(osc_type);
            // info!("set to {osc_type:?}");
        }
    }

    pub fn set_overtones(&mut self, on: bool) {
        self.overtones = on;

        // info!("overtones on: {on}")
    }

    pub fn set_note(&mut self, note: Note) {
        self.note = note;
        self.osc.set_frequency(note.into());

        // info!("set note to {note}")
    }
}

impl Module for Vco {
    async fn get_samples(&mut self) -> Vec<(u8, Float)> {
        let sample = self.osc.get_sample() * self.volume_in;
        // info!("sample {sample}");
        vec![(0, sample)]
    }

    async fn recv_samples(&mut self, input_n: u8, samples: &[Float]) {
        if input_n == PITCH_INPUT {
            self.osc.set_frequency(samples[0]);
        } else if input_n == VOLUME_INPUT {
            self.volume_in = samples.iter().sum::<Float>().tanh();
        } else if input_n == PITCH_BEND_INPUT {
            self.osc.apply_bend(samples.iter().sum::<Float>().tanh());
        } else {
            error!("invalid input: {input_n} for VCO module");
        }
    }
}
