use crate::{
    common::Module,
    osc::{OscType, Oscilator},
    Float,
};
use tracing::*;

pub const N_INPUTS: u8 = 3;
pub const N_OUTPUTS: u8 = 2;

pub const PITCH_IN: u8 = 0;
pub const VOL_IN: u8 = 1;
pub const OSC_TYPE_IN: u8 = 2;

pub struct Lfo {
    // pub routing_table: Router,
    pub osc_type: OscType,
    /// the oscilator that produces samples
    pub osc: Oscilator,
    /// where the data from the volume input is stored
    pub volume_in: Float,
    pub id: u8,
}

impl Lfo {
    pub fn new(id: u8) -> Self {
        let osc_type = OscType::Sine;
        let mut osc = Oscilator::new();
        let volume_in = 0.5;

        // DEBUG
        osc.set_frequency(2.5);
        // volume_in = 0.25;

        Self {
            osc_type,
            osc,
            volume_in,
            id,
        }
    }

    pub fn set_osc_type(&mut self, osc_type: OscType) {
        if osc_type != self.osc_type {
            self.osc_type = osc_type;
            self.osc.set_waveform(osc_type);
            info!("set to {osc_type:?}");
        }
    }

    pub fn set_pitch(&mut self, pitch: Float) {
        self.osc.set_frequency(pitch);
    }
}

impl Module for Lfo {
    fn get_samples(&mut self) -> Vec<(u8, Float)> {
        let sample = self.osc.get_sample() * self.volume_in;
        // info!("lfo => {sample}");

        vec![(0, sample), (1, sample * -1.0)]
    }

    fn recv_samples(&mut self, input_n: u8, samples: &[Float]) {
        if input_n == PITCH_IN {
            self.osc.set_frequency(samples[0]);
        } else if input_n == VOL_IN {
            self.volume_in = (samples.iter().sum::<Float>().tanh() + 1.0) * 0.5;
        } else if input_n == OSC_TYPE_IN {
            error!("can not yet set LFO oscillator type");
        } else {
            error!("invalid input: {input_n} for LFO module");
        }
    }
}
