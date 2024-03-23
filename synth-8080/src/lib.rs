#![feature(exclusive_range_pattern, let_chains)]
pub use lib::{Float, SAMPLE_RATE};
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
