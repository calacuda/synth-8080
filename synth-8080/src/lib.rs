use anyhow::{Result, bail};
use common::Module;
use lib::ModuleType;
pub use lib::{Float, SAMPLE_RATE};
use log::error;
// use output::Audio;
use rodio::{OutputStream, Sink, Source};
use std::{future::Future, mem::size_of, sync::Arc, task::Poll};
pub use tokio::spawn;
use tracing::*;

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
pub mod midi_osc;
pub mod osc;
pub mod output;
pub mod overdrive;
// pub mod poly_midi_osc;
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

        // info!("locking controller.modules");

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

        // info!("locking controller.connections");
        for con in self.controller.connections.lock().unwrap().iter() {
            dest_samples[con.dest_module as usize][con.dest_input as usize] +=
                src_samples[con.src_module as usize][con.src_output as usize];

            let dest = (con.dest_module, con.dest_input);

            if !destinations.contains(&dest) {
                destinations.push(dest);
            }
        }

        // info!("built destinations");

        for (dest_mod, dest_in) in destinations {
            let sample = dest_samples[dest_mod as usize][dest_in as usize];

            if dest_mod == 0 {
                self.controller
                    .output
                    .lock()
                    .unwrap()
                    .recv_samples(dest_in, &vec![sample]);
            } else {
                self.controller.modules.lock().unwrap().send_sample_to(
                    dest_mod as usize,
                    dest_in as usize,
                    &vec![sample],
                );
            }
        }

        // info!("sent samples");

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

pub fn default_modules() -> Vec<ModuleType> {
    [
        // ModuleType::Output,
        ModuleType::MCO,
        ModuleType::Lfo,
        ModuleType::Echo,
        ModuleType::Chorus,
        ModuleType::Delay,
        ModuleType::OverDrive,
        ModuleType::Reverb,
        ModuleType::Lfo,
        ModuleType::Lfo,
        ModuleType::Lfo,
    ]
    .to_vec()
}

pub async fn mk_synth(
    modules: &[ModuleType],
) -> Result<(
    Arc<controller::Controller>,
    (Sink, impl Source<Item = f32> + Iterator<Item = f32> + use<>),
)> {
    // TODO: read config

    // let (raw_ctrlr, _audio_handle) = controller::Controller::new(&modules).await.map_or_else(
    let (raw_ctrlr, audio_handle) = controller::Controller::new(modules).await.map_or_else(
        |e| {
            error!("{e}");
            bail!(e);
        },
        |c| Ok(c),
    )?;
    let ctrlr = Arc::new(raw_ctrlr);

    let sink = if let Ok(output) = ctrlr.output.lock() {
        rodio::Sink::connect_new(&output.stream.mixer())
    } else {
        bail!("failed to start audio playback")
    };

    info!("{} modules made", modules.len());

    info!(
        "synth started with: {} bit float samples, and a sample rate of {}",
        size_of::<Float>() * 8,
        SAMPLE_RATE
    );

    Ok((ctrlr, (sink, audio_handle)))
}

// TODO: overhaul this to use MCO instead of VCO
pub fn default_connections(synth: Arc<controller::Controller>, n: usize) {
    let mods = synth.modules.lock().unwrap();

    // get chorus
    let chorus_i = mods
        .indices
        .iter()
        .enumerate()
        .filter(|x| x.1.0 == ModuleType::Chorus)
        .next()
        .unwrap()
        .0 as u8
        + 1;
    info!("chorus_i => {chorus_i}");
    let mut vco_i_s = mods
        .indices
        .iter()
        .enumerate()
        .filter(|x| x.1.0 == ModuleType::Vco);
    let mut env_i_s = mods
        .indices
        .iter()
        .enumerate()
        .filter(|x| x.1.0 == ModuleType::EnvFilter);

    for _i in 0..n {
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
