use crate::{
    common::{Module, ModuleType},
    Float,
};
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
    // pub reverb: Vec<(Vec<Input>, Vec<Output>)>,
    // pub mid_pass: Vec<(Vec<Input>, Vec<Output>)>,
    // pub gain: Vec<(Vec<Input>, Vec<Output>)>,
    // pub delay: Vec<(Vec<Input>, Vec<Output>)>,
    // pub chorus: Vec<(Vec<Input>, Vec<Output>)>,
    // pub audio_in: Vec<(Vec<Input>, Vec<Output>)>,
    /// allows for easier indexing into this struct. the index of the items in this Vec corespond
    /// to the modules ID
    pub indeces: Vec<(ModuleType, usize)>,
}

impl Modules {
    pub async fn get_output(&mut self, id: usize) -> Option<Vec<(u8, Float)>> {
        if id == 0 {
            return Some(Vec::new());
        }

        let (mod_type, i) = self.indeces.get(id - 1)?;
        // info!("({mod_type:?}, {i})");
        // info!("n vcos {}", self.vco.len());

        Some(match mod_type {
            ModuleType::Vco => self.vco[*i].get_samples().await,
            ModuleType::Lfo => self.lfo[*i].get_samples().await,
            ModuleType::EnvFilter => self.filter[*i].get_samples().await,
            ModuleType::Echo => self.echo[*i].get_samples().await,
            _ => {
                error!("{mod_type:?} is not yet in Modules.get_output(...)'s match statement. pls fix that");
                return None;
            }
        })
    }

    pub async fn send_sample_to(&mut self, id: usize, input: usize, samples: &[Float]) {
        if id == 0 {
            warn!("break");
            // self.output.recv_samples(0, samples);
            return;
        }

        let (mod_type, i) = self.indeces[id - 1];

        match mod_type {
            ModuleType::Vco => self.vco[i].recv_samples(input as u8, samples).await,
            ModuleType::Lfo => self.lfo[i].recv_samples(input as u8, samples).await,
            ModuleType::EnvFilter => self.filter[i].recv_samples(input as u8, samples).await,
            ModuleType::Echo => self.echo[i].recv_samples(input as u8, samples).await,
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

        iter.into_iter().for_each(|mod_type| match mod_type {
            ModuleType::Vco => {
                s.vco.push(crate::vco::Vco::new((s.indeces.len()) as u8));
                s.indeces.push((*mod_type, s.vco.len() - 1));
            }
            ModuleType::Lfo => {
                s.lfo.push(crate::lfo::Lfo::new((s.indeces.len()) as u8));
                s.indeces.push((*mod_type, s.lfo.len() - 1));
            }
            ModuleType::EnvFilter => {
                s.filter.push(crate::envelope::EnvelopeFilter::new(
                    (s.indeces.len() - 1) as u8,
                ));
                s.indeces.push((*mod_type, s.filter.len() - 1));
            }
            ModuleType::Echo => {
                s.echo.push(crate::echo::Echo::new((s.indeces.len()) as u8));
                s.indeces.push((*mod_type, s.echo.len() - 1));
            }
            _ => {
                error!(
                    "{mod_type:?} is not yet in Modules.from(...)'s match statement. pls fix that"
                );
            }
        });

        s
    }
}
