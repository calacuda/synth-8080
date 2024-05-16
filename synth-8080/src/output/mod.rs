// use crate::spawn;
use crate::{common::Module, Float, JoinHandle, SAMPLE_RATE};
use crossbeam_channel::{unbounded, Receiver, Sender};
use rodio::{OutputStream, OutputStreamHandle, Source};
use tracing::*;
// use tokio::spawn;

// TODO: Add a volume input to this
pub const N_INPUTS: u8 = 1;
pub const N_OUTPUTS: u8 = 0;

#[derive(Clone)]
pub struct Audio {
    ext_sync: Sender<()>,
    int_sync: Receiver<Float>,
}

impl Audio {
    pub fn new(ext_sync: Sender<()>, int_sync: Receiver<Float>) -> Self {
        Self { ext_sync, int_sync }
    }
}

impl Iterator for Audio {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // info!("beginning of next.");
        self.ext_sync.send(()).unwrap();
        let sample = self.int_sync.try_recv().unwrap_or(0.0);
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
        // 48_000
        // 44_100
        // 22_050
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

pub struct Output {
    /// used for internal synchronization with the audio buffer sent to rodio
    int_sync: Sender<Float>,
    /// the current sample
    sample: Float,
    /// the rodio output stream, it itsn't used but must never be dropped else audio output will cease
    _stream: OutputStream,
}

impl Output {
    pub fn new(ext_sync: Sender<()>) -> (Self, (OutputStreamHandle, Audio)) {
        info!("making audio output struct");
        let sample = 0.0;
        let (int_sync, rx) = unbounded();
        let audio = Audio::new(ext_sync, rx);
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        info!("playing audio struct");

        (
            Self {
                // audio,
                // ext_sync,
                int_sync,
                sample,
                _stream,
            },
            // spawn(async move {
            //     stream_handle.play_raw(audio).unwrap();
            // }),
            (stream_handle, audio),
        )
    }
}

impl Module for Output {
    fn get_samples(&mut self) -> Vec<(u8, Float)> {
        vec![(0, self.sample)]
    }

    fn recv_samples(&mut self, _input_n: u8, samples: &[Float]) {
        let sample: Float = samples.iter().sum();
        self.sample = sample.tanh();
        // warn!("sample -> {sample}");

        if let Err(e) = self.int_sync.send(self.sample) {
            error!("could not send sample to Audio struct. got error: {e}");
        };
    }
}

unsafe impl Sync for Output {}
unsafe impl Send for Output {}
