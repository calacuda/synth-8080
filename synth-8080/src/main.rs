#![feature(exclusive_range_pattern)]
use anyhow::{bail, Result};
use common::ModuleType;
use std::mem;
use tokio::join;
use tracing::{error, info, Level};

pub use tokio::task::spawn;
pub type JoinHandle = tokio::task::JoinHandle<()>;
// pub use std::thread::spawn;
// pub type JoinHandle = std::thread::JoinHandle<()>;
pub use tokio::sync::mpsc::channel;
// pub type channel = tokio::sync::

// pub mod adbdr;
// pub mod adsr;
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
pub mod reverb;
pub mod router;
pub mod vco;

// pub type Float = f32;
pub type Float = f64;
pub const SAMPLE_RATE: u32 = 48_000;
pub const FLOAT_LEN: usize = mem::size_of::<Float>();

#[tokio::main(flavor = "multi_thread", worker_threads = 30)]
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
    // TODO: have the controller act as a router to route output from modules to connected inputs
    // TODO: try spinning up each module as its own OS thread and use unix-sockets to handle
    // message passing
    // TODO: test WEBSOCKETS But with the input requesting data from the output, that way every
    // things stays synced up
    // TODO: instead of having every input be its own channel have one input channel per module and
    // add addressing. ALSO have output controlled by controller.
    // TODO: make it syncronous (just to check to see if its efficient enough).
    let modules = [
        // ModuleType::Output,
        ModuleType::Vco,
        ModuleType::EnvFilter,
        ModuleType::Lfo,
        ModuleType::Echo,
        ModuleType::Echo,
        ModuleType::Vco,
        ModuleType::Vco,
        ModuleType::Vco,
        ModuleType::EnvFilter,
        ModuleType::EnvFilter,
        ModuleType::EnvFilter,
    ];

    let (mut ctrlr, audio_handle) = controller::Controller::new(&modules).await.map_or_else(
        |e| {
            error!("{e}");
            bail!(e);
        },
        |c| Ok(c),
    )?;
    info!("{} modules made", modules.len());
    // TODO: test changing envelopes

    // *** test trem & vibrato *** //
    // ctrlr.connect(1, 0, 0, 0)?;
    // // connect LFO to VCO volume input
    // ctrlr.connect(3, 0, 1, vco::VOLUME_INPUT)?;
    // // ctrlr.connect(2, 0, 1, vco::PITCH_BEND_INPUT)?;
    // sleep(Duration::from_secs_f64(1.0)).await;
    // ctrlr.connect(2, 0, 1, vco::VOLUME_INPUT)?;
    // // ctrlr.connect(2, 0, 1, vco::PITCH_BEND_INPUT)?;
    // sleep(Duration::from_secs_f64(2.0)).await;
    // info!("disconnecting trem");
    // ctrlr.disconnect(2, 0, 1, vco::VOLUME_INPUT)?;
    // sleep(Duration::from_secs_f64(1.0)).await;
    // ctrlr.connect(2, 0, 1, vco::PITCH_BEND_INPUT)?;

    // connect vco to output directly
    // ctrlr.connect(1, 0, 0, 0)?;
    // connect vco to adbdr
    if let Err(e) = ctrlr.connect(1, 0, 2, envelope::AUDIO_IN) {
        error!("{e}");
    };
    // connect adbdr to output
    // if let Err(e) = ctrlr.connect(2, 0, 0, 0) {
    //     error!("{e}");
    // }
    // connect adbdr to echo
    ctrlr.connect(2, 0, 4, echo::AUDIO_INPUT)?;
    // connect echo to output
    ctrlr.connect(4, 0, 0, 0)?;
    // connect vco to adsr
    // ctrlr.connect(1, 0, 5, adsr::AUDIO_IN)?;
    // connect adsr to output
    // ctrlr.connect(5, 0, 0, 0)?;

    // info!("info => {}", ctrlr.module);
    let hardware_handle = ctrlr.start_harware();
    let synth_handle = ctrlr.step();
    // stream_handle.play_raw(ctrlr.output)?;

    join!(synth_handle, hardware_handle, audio_handle);
    // sleep(Duration::from_secs(2)).await;

    // warn!("about to stop syntheses");

    // info!("syntheses stopped");

    Ok(())
}
