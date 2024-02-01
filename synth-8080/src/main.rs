use anyhow::Result;
use common::ModuleType;
use std::mem;
use tracing::info;

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
    let modules = [ModuleType::Vco, ModuleType::Output];

    let ctrlr = controller::Controller::new(&modules).await?;
    // info!("{}", ctrlr.modules.lock().unwrap().len());
    ctrlr.connect(0, 0, 1, 0)?;
    ctrlr.start().await?;

    ctrlr.handles.iter().for_each(|handle| handle.abort());

    Ok(())
}
