use crate::{
    common::Module,
    envelope::{self, adbdr, adsr, EnvelopeFilter, Filter, FILTER_OPEN_IN},
    vco::{self, Vco, PITCH_BEND_INPUT},
};
use anyhow::{bail, Result};
use lib::{notes::Note, FilterType, Float, OscType};
use std::ops::IndexMut;
use tracing::*;

pub const N_INPUTS: u8 = envelope::N_INPUTS + vco::N_INPUTS;
pub const N_OUTPUTS: u8 = 1;
// pub const PITCH_BEND: u8 = 9;
// pub const VOLUME: u8 = 8;

pub struct MidiOsc {
    pub oscs: Vec<(Vco, EnvelopeFilter)>,
    /// how many vcos/envs there are
    size: usize,
    notes: Vec<Option<Note>>,
    pub overtones: bool,
    // TODO: add a single vco/env combo to be controlled with signal inputs
}

impl Default for MidiOsc {
    fn default() -> Self {
        Self::new(10)
    }
}

impl MidiOsc {
    pub fn new(size: usize) -> Self {
        let oscs = (0..size).into_iter().map(|i| {
            // trace!("made {i}, oscillator filter combos");
            (Vco::new(i as u8), EnvelopeFilter::new(i as u8))
        });
        trace!("made oscillator and filter combos");

        let notes = (0..size).into_iter().map(|_| None);

        Self {
            oscs: oscs.collect(),
            size,
            notes: notes.collect(),
            overtones: false,
        }
    }

    pub fn set_polyphony(&mut self, n: usize) {
        self.oscs = (0..n)
            .into_iter()
            .map(|i| (Vco::new(i as u8), EnvelopeFilter::new(i as u8)))
            .collect();

        self.notes = (0..n).into_iter().map(|_| None).collect();
        self.size = n;
    }

    pub fn set_overtones(&mut self, on: bool) {
        self.overtones = on;
        self.oscs
            .iter_mut()
            .for_each(|(vco, _env)| vco.set_overtones(on));
    }

    pub fn is_playing(&mut self, note: Note) -> bool {
        self.notes.contains(&Some(note))
    }

    pub fn play_note(&mut self, note: Note) -> Result<()> {
        if self.notes.contains(&Some(note)) {
            bail!("{note} is already being played.");
        }
        // else {
        //     debug!("playing => {note}");
        // }

        for i in 0..self.size {
            if self.notes[i].is_none() {
                self.notes[i] = Some(note);

                let (ref mut vco, ref mut env) = self.oscs.index_mut(i);

                vco.set_note(note);
                env.recv_samples(FILTER_OPEN_IN, &vec![1.0]);

                return Ok(());
            }
        }

        bail!("no free oscilators");
    }

    pub fn stop_note(&mut self, note: Note) -> Result<()> {
        if !self.notes.contains(&Some(note)) {
            bail!("{note} is not being played.");
        }

        for i in 0..self.size {
            if self.notes[i] == Some(note) {
                self.notes[i] = None;

                let (vco, ref mut env) = self.oscs.index_mut(i);

                // vco.osc.set_frequency(0.0);
                env.recv_samples(FILTER_OPEN_IN, &vec![0.0]);

                return Ok(());
            }
        }

        bail!("note not found, or an unknown error ocured...");
    }

    pub fn set_wave_form(&mut self, wave_form: OscType) {
        self.oscs
            .iter_mut()
            .for_each(|(vco, _env)| vco.set_osc_type(wave_form));
    }

    pub fn set_env(&mut self, filter_type: FilterType) {
        self.oscs
            .iter_mut()
            .for_each(|(_vco, env)| env.set_filter_type(filter_type));
    }

    /// sets volume; inpute is assumed to be between 0 and 1.0
    pub fn set_volume(&mut self, volume: Float) {
        self.oscs
            .iter_mut()
            .for_each(|(vco, _env)| vco.osc.volume = volume);
    }

    pub fn set_attack(&mut self, atk: Float) {
        self.oscs
            .iter_mut()
            .for_each(|(_vco, env)| env.recv_samples(4, &vec![atk]));
    }

    pub fn set_decay(&mut self, decay: Float) {
        self.oscs
            .iter_mut()
            .for_each(|(_vco, env)| env.recv_samples(5, &vec![decay]));
    }

    pub fn set_sustain(&mut self, threshold: Float) {
        if self.oscs[0].1.filter_type == FilterType::ADSR {
            self.oscs
                .iter_mut()
                .for_each(|(_vco, env)| env.recv_samples(6, &vec![threshold]));
        }
    }

    pub fn set_break(&mut self, threshold: Float) {
        // if self.oscs[0].1.filter_type == FilterType::ADBDR {
        //     self.oscs
        //         .iter_mut()
        //         .for_each(|(_vco, env)| env.recv_samples(adbdr::DECAY_THRESHOLD, &vec![threshold]));
        // }
    }

    pub fn set_decay_2(&mut self, threshold: Float) {
        // if self.oscs[0].1.filter_type == FilterType::ADBDR {
        //     self.oscs
        //         .iter_mut()
        //         .for_each(|(_vco, env)| env.recv_samples(adbdr::DECAY_2_IN, &vec![threshold]));
        // }
    }

    pub fn set_cutoff(&mut self, value: Float) {
        self.oscs
            .iter_mut()
            // .for_each(|(_vco, env)| env.allpass.set_cutoff(value));
            .for_each(|(_vco, env)| env.filter.set_cutoff(value));
    }

    pub fn set_resonance(&mut self, value: Float) {
        self.oscs
            .iter_mut()
            // .for_each(|(_vco, env)| env.allpass.set_resonance(value));
            .for_each(|(_vco, env)| env.filter.set_resonance(value));
    }
}

impl Module for MidiOsc {
    fn recv_samples(&mut self, input_n: u8, samples: &[lib::Float]) {
        if input_n < envelope::N_INPUTS {
            self.oscs
                .iter_mut()
                .for_each(|(_vco, env)| env.recv_samples(input_n, samples));
        } else {
            let input = input_n - envelope::N_INPUTS;

            if input >= vco::N_INPUTS {
                error!("{input_n} => {input}, is an invalid input number for the MidiOsc (MCO)");
                return;
            };

            self.oscs
                .iter_mut()
                .for_each(|(vco, _env)| vco.recv_samples(input, samples));
        }
    }

    fn get_samples(&mut self) -> Vec<(u8, lib::Float)> {
        let raw_sample: Float = self
            .oscs
            .iter_mut()
            .filter_map(|(vco, env)| {
                if env.is_pressed() {
                    let sample: Float = vco
                        .get_samples()
                        .into_iter()
                        .map(|(output, sample)| sample)
                        .sum();

                    env.recv_samples(envelope::AUDIO_IN, &vec![sample]);

                    // debug!("{} => {}", env.envelope.get_env(), sample);

                    Some(
                        env.get_samples()
                            .into_iter()
                            .filter_map(|(output, sample)| {
                                if output == envelope::AUDIO_OUT {
                                    Some(sample)
                                } else {
                                    None
                                }
                            })
                            .sum::<Float>(),
                    )
                } else {
                    // vco.osc.set_frequency(0.0);
                    None
                }
            })
            .sum();
        // .sum();
        let n_notes = self
            .notes
            .iter()
            .filter_map(|note| *note)
            .collect::<Vec<_>>()
            .len() as Float;
        // // info!("n_notes {n_notes}");
        // let sample = if n_notes > 0.0 {
        //     raw_sample * n_notes
        // } else {
        //     0.0
        // };
        // let raw_sample: Float = raw_samples.iter().sum();
        // let n_notes = raw_samples.iter().len() as Float;
        // info!("n_notes {}", n_notes * 2.0 / n_notes.exp());
        // info!("n_notes {}", 0.75 / n_notes.ln_1p());

        let sample = if n_notes > 1.0 {
            raw_sample / n_notes.sqrt()
        } else {
            raw_sample
        };

        // info!("sample {raw_sample} : {sample}");

        vec![(0, sample)]
    }

    fn get_input_names() -> impl Iterator<Item = impl std::fmt::Display> {
        let mut vco: Vec<String> = Vco::get_input_names()
            .map(|name| format!("{name}"))
            .collect();
        let mut env: Vec<String> = EnvelopeFilter::get_input_names()
            .map(|name| format!("{name}"))
            .collect();

        vco.append(&mut env);

        vco.into_iter()
    }

    fn get_output_names() -> impl Iterator<Item = impl std::fmt::Display> {
        let mut vco: Vec<String> = Vco::get_output_names()
            .map(|name| format!("{name}"))
            .collect();
        let mut env: Vec<String> = EnvelopeFilter::get_output_names()
            .map(|name| format!("{name}"))
            .collect();

        vco.append(&mut env);

        vco.into_iter()
    }
}
