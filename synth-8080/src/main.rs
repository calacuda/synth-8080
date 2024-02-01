use anyhow::Result;
use router::{Modules, Router};
use std::{
    mem,
    sync::{Arc, Mutex},
};
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

    let router: Router = Arc::new(vec![
        vec![Arc::new(Mutex::new(Vec::with_capacity(3)))],
        vec![Arc::new(Mutex::new(Vec::with_capacity(1)))],
    ]);
    let mut modules = Modules::default();

    let (oscilator, handle) = vco::start(router.clone()).await?;
    oscilator.connect_to(0, 1, 0).await?;
    oscilator.set_note(common::notes::Note::A4);
    modules.vco.push(oscilator);
    // TODO: trun led red

    let (_stream, _audio_handle) = output::start(router, &mut modules).await?;

    handle.await?;
    Ok(())
}
