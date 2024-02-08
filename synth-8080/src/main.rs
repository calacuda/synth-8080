use anyhow::{bail, Result};
use common::ModuleType;
use controller::EnvelopeType;
use std::mem;
use tracing::{error, info, warn};

pub mod adbdr;
pub mod adsr;
pub mod audio_in;
pub mod chorus;
pub mod common;
pub mod controller;
pub mod delay;
pub mod echo;
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

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_thread_ids(true)
        .with_target(true)
        .without_time()
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber)?;

    info!("synth begin");

    // TODO: read config
    let modules = [
        ModuleType::Output,
        ModuleType::Vco,
        ModuleType::Lfo,
        ModuleType::Adbdr,
        ModuleType::Echo,
        ModuleType::Adsr,
    ];

    let ctrlr = controller::Controller::new(&modules).await.map_or_else(
        |e| {
            error!("{e}");
            bail!(e);
        },
        |c| Ok(c),
    )?;
    info!("{} modules made", ctrlr.modules.lock().unwrap().len());
    {
        let mut filter = ctrlr.envelope_type.lock().unwrap();
        *filter = EnvelopeType::ADSR;
    }

    // *** test trem & vibrato *** //
    // ctrlr.connect(1, 0, 0, 0)?;
    // // connect LFO to VCO volume input
    // // ctrlr.connect(2, 0, 1, vco::VOLUME_INPUT)?;
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
    ctrlr.connect(1, 0, 0, 0)?;
    // connect vco to adbdr
    ctrlr.connect(1, 0, 3, adbdr::AUDIO_IN)?;
    // connect adbdr to output
    // ctrlr.connect(3, 0, 0, 0)?;
    // connect adbdr to echo
    // ctrlr.connect(3, 0, 4, echo::AUDIO_INPUT)?;
    // connect echo to output
    // ctrlr.connect(4, 0, 0, 0)?;
    // connect vco to adsr
    ctrlr.connect(1, 0, 5, adsr::AUDIO_IN)?;
    // connect adsr to output
    ctrlr.connect(5, 0, 0, 0)?;

    // info!("info => {}", ctrlr.module);
    ctrlr.start().await?;
    // sleep(Duration::from_secs(2)).await;

    warn!("about to stop syntheses");
    ctrlr
        .handles
        .lock()
        .unwrap()
        .iter()
        .for_each(|handle| handle.abort());
    info!("syntheses stopped");

    Ok(())
}
