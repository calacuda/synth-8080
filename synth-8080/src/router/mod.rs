use crate::{common::Module, Float};
use lib::ModuleType;
use tracing::*;

#[derive(Default)]
pub struct Modules {
    /// a list of the echo modules
    pub echo: Vec<crate::echo::Echo>,
    /// a list of the LFOs
    pub lfo: Vec<crate::lfo::Lfo>,
    /// a list of the VCOs
    pub vco: Vec<crate::vco::Vco>,
    /// a list of evnvelope filters
    pub filter: Vec<crate::envelope::EnvelopeFilter>,
    pub reverb: Vec<crate::reverb::ReverbModule>,
    // pub mid_pass: Vec<(Vec<Input>, Vec<Output>)>,
    // pub gain: Vec<(Vec<Input>, Vec<Output>)>,
    pub delay: Vec<crate::delay::Delay>,
    pub chorus: Vec<crate::chorus::Chorus>,
    pub over_drive: Vec<crate::overdrive::OverDrive>,
    pub mco: Vec<crate::midi_osc::MidiOsc>,
    // pub audio_in: Vec<(Vec<Input>, Vec<Output>)>,
    /// allows for easier indexing into this struct. the index of the items in this Vec correspond
    /// to the modules ID
    pub indices: Vec<(ModuleType, usize)>,
}

impl Modules {
    pub fn get_output(&mut self, id: usize) -> Option<Vec<(u8, Float)>> {
        if id == 0 {
            return Some(Vec::new());
        }

        let (mod_type, i) = self.indices.get(id - 1)?;
        // info!("({mod_type:?}, {i})");
        // info!("n vcos {}", self.vco.len());

        Some(match mod_type {
            ModuleType::Vco => self.vco[*i].get_samples(),
            ModuleType::Lfo => self.lfo[*i].get_samples(),
            ModuleType::EnvFilter => self.filter[*i].get_samples(),
            ModuleType::Echo => self.echo[*i].get_samples(),
            ModuleType::Chorus => self.chorus[*i].get_samples(),
            ModuleType::Delay => self.delay[*i].get_samples(),
            ModuleType::OverDrive => self.over_drive[*i].get_samples(),
            ModuleType::Reverb => self.reverb[*i].get_samples(),
            ModuleType::MCO => self.mco[*i].get_samples(),
            _ => {
                error!("{mod_type:?} is not yet in Modules.get_output(...)'s match statement. pls fix that");
                return None;
            }
        })
    }

    pub fn send_sample_to(&mut self, id: usize, input: usize, samples: &[Float]) {
        if id == 0 {
            warn!("break");
            // self.output.recv_samples(0, samples);
            return;
        }

        let (mod_type, i) = self.indices[id - 1];

        match mod_type {
            ModuleType::Vco => self.vco[i].recv_samples(input as u8, samples),
            ModuleType::Lfo => self.lfo[i].recv_samples(input as u8, samples),
            ModuleType::EnvFilter => self.filter[i].recv_samples(input as u8, samples),
            ModuleType::Echo => self.echo[i].recv_samples(input as u8, samples),
            ModuleType::Chorus => self.chorus[i].recv_samples(input as u8, samples),
            ModuleType::Delay => self.delay[i].recv_samples(input as u8, samples),
            ModuleType::OverDrive => self.over_drive[i].recv_samples(input as u8, samples),
            ModuleType::Reverb => self.reverb[i].recv_samples(input as u8, samples),
            ModuleType::MCO => self.mco[i].recv_samples(input as u8, samples),
            _ => {
                error!("{mod_type:?} is not yet in Modules.get_output(...)'s match statement. pls fix that");
                return;
            }
        }
    }
}

// impl FromIterator<ModuleType> for Modules {
//     fn from_iter<I: IntoIterator<Item = ModuleType>>(iter: I) -> Self {
impl From<&[ModuleType]> for Modules {
    fn from(iter: &[ModuleType]) -> Self {
        let mut s = Self::default();

        iter.into_iter().for_each(|mod_type| {
            // trace!("making a {mod_type:?} module");
            match mod_type {
                ModuleType::Vco => {
                    s.vco.push(crate::vco::Vco::new((s.indices.len()) as u8));
                    s.indices.push((*mod_type, s.vco.len() - 1));
                }
                ModuleType::Lfo => {
                    s.lfo.push(crate::lfo::Lfo::new((s.indices.len()) as u8));
                    s.indices.push((*mod_type, s.lfo.len() - 1));
                }
                ModuleType::EnvFilter => {
                    s.filter.push(crate::envelope::EnvelopeFilter::new(
                        (s.indices.len() - 1) as u8,
                    ));
                    s.indices.push((*mod_type, s.filter.len() - 1));
                }
                ModuleType::Echo => {
                    s.echo.push(crate::echo::Echo::new((s.indices.len()) as u8));
                    s.indices.push((*mod_type, s.echo.len() - 1));
                }
                ModuleType::Chorus => {
                    s.chorus
                        .push(crate::chorus::Chorus::new((s.indices.len()) as u8));
                    s.indices.push((*mod_type, s.chorus.len() - 1));
                }
                ModuleType::Delay => {
                    s.delay
                        .push(crate::delay::Delay::new((s.indices.len()) as u8));
                    s.indices.push((*mod_type, s.delay.len() - 1));
                }
                ModuleType::OverDrive => {
                    s.over_drive.push(crate::overdrive::OverDrive::new());
                    s.indices.push((*mod_type, s.over_drive.len() - 1));
                }
                ModuleType::Reverb => {
                    s.reverb.push(crate::reverb::ReverbModule::new());
                    s.indices.push((*mod_type, s.reverb.len() - 1));
                }
                ModuleType::MCO => {
                    s.mco.push(crate::midi_osc::MidiOsc::default());
                    s.indices.push((*mod_type, s.mco.len() - 1));
                }
                _ => {
                    error!(
                        "{mod_type:?} is not yet in Modules.from(...)'s match statement. pls fix that"
                    );
                }
            }
        });

        s
    }
}
