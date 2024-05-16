#![feature(exclusive_range_pattern, let_chains)]
use anyhow::{bail, Result};
use common::Module;
use common::ModuleType;
// pub use lib::{Float, SAMPLE_RATE};
pub use lib::{Float, SAMPLE_RATE};
use log::error;
use output::Audio;
use rodio::OutputStreamHandle;
use std::{future::Future, sync::Arc, task::Poll};
// use tokio::task::spawn;
use tracing::*;

// #[cfg(feature("hardware"))]
pub use tokio::spawn;

// #[cfg(feature("hardware"))]

pub type JoinHandle = tokio::task::JoinHandle<()>;

pub mod audio_in;
pub mod chorus;
pub mod common;
pub mod controller;
pub mod delay;
pub mod echo;
pub mod envelope;
pub mod gain;
pub mod lfo;
pub mod mid_pass;
pub mod osc;
pub mod output;
pub mod overdrive;
pub mod reverb;
pub mod router;
pub mod vco;

pub struct AudioGen {
    pub controller: Arc<controller::Controller>,
}

impl Future for AudioGen {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        // self.deref().controller.step();
        // info!("waiting on sync signal");

        if let Err(e) = self.controller.sync.recv() {
            error!("error receiving sync message: {e}");
        };

        let mut src_samples = [[0.0; 16]; u8::MAX as usize];

        for src in 0..u8::MAX as usize {
            if let Some(mods) = self.controller.modules.lock().unwrap().get_output(src) {
                // println!("foobar");
                mods.into_iter()
                    .for_each(|(output, sample)| src_samples[src][output as usize] += sample);
            } else {
                break;
            }
        }

        let mut dest_samples = [[0.0; 16]; u8::MAX as usize];
        let mut destinations: Vec<(u8, u8)> = Vec::with_capacity(256);

        for con in self.controller.connections.lock().unwrap().iter() {
            dest_samples[con.dest_module as usize][con.dest_input as usize] +=
                src_samples[con.src_module as usize][con.src_output as usize];

            let dest = (con.dest_module, con.dest_input);

            if !destinations.contains(&dest) {
                destinations.push(dest);
            }
        }

        for (dest_mod, dest_in) in destinations {
            let sample = dest_samples[dest_mod as usize][dest_in as usize];

            if dest_mod == 0 {
                self.controller
                    .output
                    .lock()
                    .unwrap()
                    .recv_samples(0, &vec![sample]);
            } else {
                self.controller.modules.lock().unwrap().send_sample_to(
                    dest_mod as usize,
                    dest_in as usize,
                    &vec![sample],
                );
            }
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

unsafe impl Send for AudioGen {}

pub fn start_logging() -> Result<()> {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_thread_ids(true)
        .with_target(true)
        .with_level(true)
        .with_max_level(Level::TRACE)
        .without_time()
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

pub async fn mk_synth() -> Result<(Arc<controller::Controller>, (OutputStreamHandle, Audio))> {
    // TODO: read config
    let modules = [
        // ModuleType::Output,
        ModuleType::Vco,
        ModuleType::EnvFilter,
        ModuleType::Lfo,
        ModuleType::Echo,
        ModuleType::Chorus,
        ModuleType::Delay, // same as echo
        ModuleType::OverDrive,
        ModuleType::Reverb,
        ModuleType::Vco,
        ModuleType::Vco,
        ModuleType::Vco,
        ModuleType::EnvFilter,
        ModuleType::EnvFilter,
        ModuleType::EnvFilter,
        ModuleType::Vco,
        ModuleType::EnvFilter,
        ModuleType::Vco,
        ModuleType::EnvFilter,
        ModuleType::Vco,
        ModuleType::EnvFilter,
        ModuleType::Vco,
        ModuleType::EnvFilter,
        ModuleType::Vco,
        ModuleType::EnvFilter,
        ModuleType::Vco,
        ModuleType::EnvFilter,
    ];

    // let (raw_ctrlr, _audio_handle) = controller::Controller::new(&modules).await.map_or_else(
    let (raw_ctrlr, audio_handle) = controller::Controller::new(&modules).await.map_or_else(
        |e| {
            error!("{e}");
            bail!(e);
        },
        |c| Ok(c),
    )?;
    let ctrlr = Arc::new(raw_ctrlr);
    info!("{} modules made", modules.len());

    Ok((ctrlr, audio_handle))
}

pub fn default_connections(synth: Arc<controller::Controller>, n: usize) {
    let mods = synth.modules.lock().unwrap();

    // get chorus
    let chorus_i = mods
        .indices
        .iter()
        .enumerate()
        .filter(|x| x.1 .0 == ModuleType::Chorus)
        .next()
        .unwrap()
        .0 as u8
        + 1;
    info!("chorus_i => {chorus_i}");
    let mut vco_i_s = mods
        .indices
        .iter()
        .enumerate()
        .filter(|x| x.1 .0 == ModuleType::Vco);
    let mut env_i_s = mods
        .indices
        .iter()
        .enumerate()
        .filter(|x| x.1 .0 == ModuleType::EnvFilter);

    for i in 0..n {
        // get vco
        if let Some((vco_i, _)) = vco_i_s.next() {
            // get env
            if let Some((env_i, _)) = env_i_s.next() {
                // connect vco to env
                _ = synth.connect(vco_i as u8 + 1, 0, env_i as u8 + 1, envelope::AUDIO_IN);

                // connect env to chorus
                _ = synth.connect(env_i as u8 + 1, 0, chorus_i, chorus::AUDIO_INPUT);
            } else {
                error!("not enopugh Envelope Filters")
            }
        } else {
            error!("not enough VCOs");
        };
    }

    // connect chorus to the output
    _ = synth.connect(chorus_i, 0, 0, 0);
}
