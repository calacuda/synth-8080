use crate::{
    common::{Connection, Module},
    router::{router_read_sample, router_send_sync, Router},
    Float,
};
use rodio::{OutputStream, Source};
use std::{ops::Deref, thread::sleep, time::Duration};
use tokio::spawn;
use tracing::info;

// TODO: Add a volume input to output
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
        let input = &self.router.deref().0[self.id][0].input;
        let ins = &self.router.deref().0[self.id][0];
        // router_send_sync(input);
        // // info!("sync sent");
        // let sample = router_read_sample(input);
        // // info!("sample => {sample}");
        let n_cons = ins.active_connections.lock().unwrap();
        let sample: Float = (0..(*n_cons) as usize)
            .map(|_i| {
                router_send_sync(&input);
                // info!("reading sample");
                // read sample from connection
                router_read_sample(&input)
            })
            .sum();
        // info!("sample => {sample}");

        Some(sample.tanh() as f32)
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
        info!("Output started");
        let audio = self.audio.clone();

        Ok(spawn(async move {
            // let delay = sleep(Duration::from_nanos(1));
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let res = stream_handle.play_raw(audio);
            // delay.await;

            info!("stream result: {res:?}");
            info!("playing generated audio");

            loop {
                sleep(Duration::from_secs(1));
            }

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
    pub async fn new(router_table: Router, mod_id: u8) -> Self {
        let audio = Audio::new(router_table, mod_id as usize);
        info!("made audio struct to handle audio out");

        Self { audio }
    }
}

// pub async fn start(inputs: ModuleInRX) -> Result<((OutputStream, OutputStreamHandle), Audio)> {
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
