// use crate::

use crate::{
    common::Module,
    envelope::{self, adbdr, adsr, EnvelopeFilter, FILTER_OPEN_IN},
    vco::{self, Vco},
};
use anyhow::{bail, Result};
use lib::{notes::Note, FilterType, Float, OscType};
use std::ops::IndexMut;
use tracing::*;

pub const N_INPUTS: u8 = 9;
pub const PITCH_BEND: u8 = 8;
pub const VOLUME: u8 = 7;

pub struct MidiOsc {
    pub oscs: Vec<(Vco, EnvelopeFilter)>,
    /// how many vcos/envs there are
    size: usize,
    notes: Vec<Option<Note>>,
    // TODO: add a single vco/env combo to be controlled with signal inputs
}

impl Default for MidiOsc {
    fn default() -> Self {
        Self::new(10)
    }
}

impl MidiOsc {
    pub fn new(size: usize) -> Self {
        let oscs = (0..size)
            .into_iter()
            .map(|i| (Vco::new(i as u8), EnvelopeFilter::new(i as u8)));

        let notes = (0..size).into_iter().map(|_| None);

        Self {
            oscs: oscs.collect(),
            size,
            notes: notes.collect(),
        }
    }

    pub fn play_note(&mut self, note: Note) -> Result<()> {
        if self.notes.contains(&Some(note)) {
            bail!("{note} is already being played.");
        }

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

                let (_vco, ref mut env) = self.oscs.index_mut(i);

                // vco.set_(0.0);
                env.recv_samples(FILTER_OPEN_IN, &vec![0.0]);

                return Ok(());
            }
        }

        bail!("note not found, unknown error ocured...");
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
            .for_each(|(_vco, env)| env.recv_samples(3, &vec![atk]));
    }

    pub fn set_decay(&mut self, decay: Float) {
        self.oscs
            .iter_mut()
            .for_each(|(_vco, env)| env.recv_samples(4, &vec![decay]));
    }

    pub fn set_sustain(&mut self, threshold: Float) {
        if self.oscs[0].1.filter_type == FilterType::ADSR {
            self.oscs
                .iter_mut()
                .for_each(|(_vco, env)| env.recv_samples(adsr::DECAY_THRESHOLD, &vec![threshold]));
        }
    }

    pub fn set_break(&mut self, threshold: Float) {
        if self.oscs[0].1.filter_type == FilterType::ADBDR {
            self.oscs
                .iter_mut()
                .for_each(|(_vco, env)| env.recv_samples(adbdr::DECAY_THRESHOLD, &vec![threshold]));
        }
    }

    pub fn set_decay_2(&mut self, threshold: Float) {
        if self.oscs[0].1.filter_type == FilterType::ADBDR {
            self.oscs
                .iter_mut()
                .for_each(|(_vco, env)| env.recv_samples(adbdr::DECAY_2_IN, &vec![threshold]));
        }
    }
}

impl Module for MidiOsc {
    fn recv_samples(&mut self, input_n: u8, samples: &[lib::Float]) {
        if input_n < 7 {
            self.oscs
                .iter_mut()
                .for_each(|(_vco, env)| env.recv_samples(input_n, samples));
        } else {
            let n = if input_n == VOLUME {
                vco::VOLUME_INPUT
            } else if input_n == PITCH_BEND {
                vco::PITCH_BEND_INPUT
            } else {
                error!("{input_n} is an invalid input_n for MidiOsc");
                return;
            };

            self.oscs
                .iter_mut()
                .for_each(|(vco, _env)| vco.recv_samples(n, samples));
        }
    }

    fn get_samples(&mut self) -> Vec<(u8, lib::Float)> {
        let sample: Float = self
            .oscs
            .iter_mut()
            .map(|(vco, env)| {
                let sample: Float = vco
                    .get_samples()
                    .into_iter()
                    .map(|(output, sample)| sample)
                    .sum();
                env.recv_samples(envelope::AUDIO_IN, &vec![sample]);

                env.get_samples()
                    .into_iter()
                    .map(|(output, sample)| sample)
                    .sum::<Float>()
            })
            .sum();

        vec![(0, sample.tanh())]
    }
}
