#![feature(exclusive_range_pattern, let_chains)]
use anyhow::{bail, Result};
use common::Module;
use common::ModuleType;
// use std::mem;
pub use lib::{Float, SAMPLE_RATE};
use std::{future::Future, sync::Arc, task::Poll};
use tokio::task::spawn;
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
pub mod osc;
pub mod output;
pub mod overdrive;
pub mod reverb;
pub mod router;
pub mod vco;

// pub type Float = f32;
// pub type Float = f64;
// pub const SAMPLE_RATE: u32 = 48_000;
// pub const FLOAT_LEN: usize = mem::size_of::<Float>();

struct AudioGen {
    controller: Arc<controller::Controller>,
}

impl Future for AudioGen {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        // self.deref().controller.step();
        if let Err(e) = self.controller.sync.recv() {
            error!("error recieving sync message: {e}");
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

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
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

    info!("synth begin");

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

    let (raw_ctrlr, _audio_handle) = controller::Controller::new(&modules).await.map_or_else(
        |e| {
            error!("{e}");
            bail!(e);
        },
        |c| Ok(c),
    )?;
    let ctrlr = Arc::new(raw_ctrlr);
    info!("{} modules made", modules.len());
    // TODO: test changing envelopes

    // *** test trem & vibrato *** //
    // ctrlr.connect(1, 0, 0, 0)?;
    // // connect LFO to VCO volume input
    // ctrlr.connect(3, 0, 1, vco::VOLUME_INPUT)?;
    ctrlr.modules.lock().unwrap().lfo[0].volume_in = 0.7;
    ctrlr.modules.lock().unwrap().lfo[0].set_pitch(1.0);
    ctrlr.connect(2, 0, 1, vco::PITCH_BEND_INPUT)?;
    // sleep(Duration::from_secs_f64(1.0)).await;
    // ctrlr.connect(2, 0, 1, vco::VOLUME_INPUT)?;
    // ctrlr.connect(2, 0, 1, vco::PITCH_BEND_INPUT)?;
    // sleep(Duration::from_secs_f64(2.0)).await;
    // info!("disconnecting trem");
    // ctrlr.disconnect(2, 0, 1, vco::VOLUME_INPUT)?;
    // sleep(Duration::from_secs_f64(1.0)).await;
    // ctrlr.connect(2, 0, 1, vco::PITCH_BEND_INPUT)?;

    // connect vco to output directly
    // ctrlr.connect(1, 0, 0, 0)?;
    // connect vco to adbdr
    ctrlr.connect(1, 0, 2, envelope::AUDIO_IN)?;
    // connect adbdr to output
    // ctrlr.connect(2, 0, 0, 0)?;
    // connect adbdr to echo
    // ctrlr.connect(2, 0, 4, echo::AUDIO_INPUT)?;
    // connect echo to output
    // ctrlr.connect(4, 0, 0, 0)?;
    // connect adbdr to chorus
    ctrlr.connect(2, 0, 5, chorus::AUDIO_INPUT)?;
    // connect chorus to output
    ctrlr.connect(5, 0, 0, 0)?;
    // connect adbdr to delay
    // ctrlr.connect(2, 0, 6, delay::AUDIO_INPUT)?;
    // connect delay to output
    // ctrlr.connect(6, 0, 0, 0)?;
    // connect chorus to overdrive
    // ctrlr.connect(5, 0, 7, overdrive::AUDIO_INPUT)?;
    // connect overdrive to output
    // ctrlr.connect(7, 0, 0, 0)?;
    // connect LFO to overdrive gain input
    // ctrlr.connect(3, 0, 7, overdrive::GAIN_INPUT)?;
    // connect chorus to reverb
    // ctrlr.connect(2, 0, 8, reverb::AUDIO_INPUT)?;
    // connect reverb to output
    // ctrlr.connect(8, 0, 0, 0)?;

    let audio = AudioGen {
        controller: ctrlr.clone(),
    };

    let audio_out_thread = spawn(audio);
    controller::harware::HardwareControls::new(ctrlr.clone())?.await;

    // join!(synth_handle, hardware_handle, audio_handle);
    // sleep(Duration::from_secs(2)).await;

    warn!("stopping syntheses");

    audio_out_thread.abort();

    info!("syntheses stoppedhttp://localhost/");

    Ok(())
}
