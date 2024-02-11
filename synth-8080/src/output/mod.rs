use crate::{common::Module, output, Float, JoinHandle};
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use rodio::{OutputStream, OutputStreamHandle, Source};
use std::sync::{Arc, Mutex};
use tokio::spawn;
use tracing::*;

// TODO: Add a volume input to output
pub const N_INPUTS: u8 = 1;
pub const N_OUTPUTS: u8 = 0;

#[derive(Clone)]
pub struct Audio {
    ext_sync: Sender<()>,
    int_sync: Receiver<Float>,
}

impl Audio {
    pub fn new(ext_sync: Sender<()>, int_sync: Receiver<Float>) -> Self {
        // let inputs = Arc::new(Mutex::new(Vec::new()));
        // let size = router.in_s.len() * 2;
        // // (*router.in_s)
        // //     .iter()
        // //     .flat_map(|a| a.iter())
        // //     .collect::<Vec<_>>()
        // //     .len();
        //
        // (0..size).for_each(|_i| {
        //     // warn!("initial first i: {i}");
        //
        //     if let Err(e) = sync.send(()) {
        //         error!("failed sending sync: {e}");
        //     }
        //     // warn!("initial first i: {i}");
        // });
        // // warn!("router len: {}", router.in_s.len());

        Self { ext_sync, int_sync }
    }
}

impl Iterator for Audio {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.ext_sync.send(()).unwrap();
        let sample = self.int_sync.recv().unwrap_or(0.0);
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

pub struct Output {
    // pub audio: Audio,
    // /// used to request new data from the
    // ext_sync: Sender<()>,
    /// used for internal syncronization
    int_sync: Sender<Float>,
    // recv: Receiver<Float>,
    sample: Float,
    stream: OutputStream,
}

impl Output {
    pub fn new(ext_sync: Sender<()>) -> (Self, JoinHandle) {
        info!("making audio output struct");
        let sample = 0.0;
        let (int_sync, rx) = unbounded();
        let audio = Audio::new(ext_sync, rx);
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        info!("playing audio struct");

        (
            Self {
                // audio,
                // ext_sync,
                int_sync,
                sample,
                stream,
            },
            spawn(async move {
                stream_handle.play_raw(audio).unwrap();
            }),
        )
    }
}

// TODO: make output able to pass its input transparently (so i can visualize audio out)
impl Module for Output {
    async fn get_samples(&mut self) -> Vec<(u8, Float)> {
        vec![(0, self.sample)]
    }

    async fn recv_samples(&mut self, _input_n: u8, samples: &[Float]) {
        let sample: Float = samples.iter().sum();
        self.sample = sample.tanh();
        // warn!("sample -> {sample}");

        if let Err(e) = self.int_sync.send(self.sample) {
            error!("could not send sample to Audio struct. got error: {e}");
        };
    }
}

// fn start(&self) -> anyhow::Result<JoinHandle> {
//     info!("Output started");
//     let audio = self.audio.clone();
//
//     Ok(spawn(async move {
//         // let delay = sleep(Duration::from_nanos(1));
//         let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//         let res = stream_handle.play_raw(audio);
//         // delay.await;
//
//         info!("stream result: {res:?}");
//         info!("playing generated audio");
//
//         loop {
//             sleep(Duration::from_secs(1));
//         }
//
//         // warn!("stopping audio playback");
//     }))
// }
//
// fn connect(&self, _connection: Connection) -> anyhow::Result<()> {
//     // trace!("connection => {:?}", connection);
//     // if connection.dest_input == 0 {
//     //     info!("connecting to output");
//     //     self.audio.inputs.lock().unwrap().push(connection);
//     // } else {
//     //     bail!("invalid input selection");
//     // }
//
//     bail!("invalid input selection");
//
//     // Ok(())
// }
//
// fn disconnect(&self, _connection: Connection) -> anyhow::Result<()> {
//     // if connection.dest_input == 0 {
//     //     self.audio
//     //         .inputs
//     //         .lock()
//     //         .unwrap()
//     //         .retain(|con| *con != connection);
//     // } else {
//     bail!("invalid input selection");
//     // }
//
//     // Ok(())
// }

// fn n_outputs(&self) -> u8 {
//     N_OUTPUTS
// }

// fn connections(&self) -> std::sync::Arc<std::sync::Mutex<Vec<Connection>>> {
//     // Arc::new(Mutex::new(Vec::new()))
//     self.audio.inputs.clone()
// }
// }

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
