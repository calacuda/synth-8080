use crate::{
    common::{mk_float, not_found, Connection},
    router::{Modules, Router},
    Float,
};
use anyhow::{ensure, Result};
use rodio::{OutputStream, OutputStreamHandle, Source};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};
use tracing::{error, info};

pub const N_INPUTS: u8 = 1;

#[derive(Clone)]
pub struct Audio {
    routing_table: Router,
}

impl Audio {
    pub fn new(routing_table: Router) -> Self {
        Self { routing_table }
    }
}

impl Iterator for Audio {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut ins = self.routing_table.deref()[1][0].lock().unwrap();
        let sample = (ins.iter().filter_map(|f| *f).sum::<Float>() / ins.len() as f64) as f32;
        // ins.clear();
        ins.iter_mut().for_each(|f| *f = None);
        // info!("sample => {sample}");
        // info!("ins => {ins:?}");

        Some(sample)
    }
}

impl Source for Audio {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        48_000
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

pub async fn start(
    routing_table: Router,
    modules: &mut Modules,
) -> anyhow::Result<(OutputStream, OutputStreamHandle)> {
    let audio = Audio::new(routing_table);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    stream_handle.play_raw(audio.clone())?;

    modules.output = Some(audio);

    Ok((_stream, stream_handle))
}

// pub async fn prepare() -> anyhow::Result {}
