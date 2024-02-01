use crate::{
    common::{Connection, Module},
    router::{router_read_sample, router_send_sync, ModuleInRX, Router},
};
use rodio::{OutputStream, OutputStreamHandle, Source};
use std::ops::Deref;
use tokio::spawn;
use tracing::{info, warn};

pub const N_INPUTS: u8 = 1;
pub const N_OUTPUTS: u8 = 0;

#[derive(Clone)]
pub struct Audio {
    router: Router,
    id: usize,
}

impl Audio {
    pub fn new(router: Router, id: usize) -> Self {
        Self { router, id }
    }
}

impl Iterator for Audio {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let input = &self.router.deref()[self.id][0].input;
        router_send_sync(input);
        // info!("sync sent");
        let sample = router_read_sample(input);
        // info!("sample => {sample}");

        Some(sample as f32)
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

// TODO: make output able to pass its input transparently (so i can visualize audio out)

impl Module for Output {
    fn start(&self) -> anyhow::Result<tokio::task::JoinHandle<()>> {
        let audio = self.audio.clone();

        Ok(spawn(async move {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let res = stream_handle.play_raw(audio);

            info!("stream result: {res:?}");
            info!("playing generated audio");

            loop {}

            // warn!("stopping audio playback");
        }))
    }

    fn connect(&self, _connection: Connection) -> anyhow::Result<()> {
        Ok(())
    }

    fn disconnect(&self, _connection: Connection) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct Output {
    pub audio: Audio,
}

impl Output {
    // TODO: pass router_table and something that can index it to get inputs
    pub async fn new(router_table: Router, mod_id: u8) -> Self {
        let audio = Audio::new(router_table, mod_id as usize);
        info!("made audio struct to handle audio out");

        Self { audio }
    }
}

// pub async fn start(inputs: ModuleInRX) -> Result<((OutputStream, OutputStreamHandle), Audio)> {
//     // TODO: add a record to file mode
//     let audio = Audio::new(inputs);
//     let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//     stream_handle.play_raw(audio.clone())?;
//
//     Ok(((_stream, stream_handle), audio))
// }

// pub async fn prepare() -> ModuleInfo {
//     ModuleInfo {
//         n_ins: 1,
//         n_outs: 0,
//         io: mk_module_ins(1),
//     }
// }
