// use crate::spawn;
use crate::{common::Module, Float, SAMPLE_RATE};
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use rodio::{OutputStream, OutputStreamHandle, Source};
use serialport::{SerialPort, TTYPort};
use std::{
    io::{Read, Write},
    thread::{spawn, JoinHandle},
    time::Duration,
};
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
        // self.ext_sync.send(()).unwrap();
        let sample = self.int_sync.try_recv().unwrap_or(0.0);
        // #[cfg(not(feature = "hardware"))]
        self.ext_sync.send(()).unwrap();
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

pub struct HWAudio {
    ext_sync: Sender<()>,
    recv: Receiver<Float>,
    serial: Option<TTYPort>,
}

impl Iterator for HWAudio {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.recv.try_recv().unwrap_or(0.0);
        // info!("sample :  {sample}");

        self.ext_sync.send(()).unwrap();
        // let serial = self.serial.as_mut().cloned();

        let ser = self.serial.as_mut();
        // spawn(move || {
        let mut sample_bytes: Vec<u8> = ((sample as f64 * i32::MAX as f64) as i32)
            .to_be_bytes()
            .into_iter()
            .collect();

        // serial.read_exact(&mut buf);

        ser.unwrap().write(&sample_bytes[..4]);
        // });

        // Some(sample as f32)
        Some(sample as f32)
    }
}

impl Source for HWAudio {
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

impl HWAudio {
    pub fn new(serial: Option<TTYPort>, ext_sync: Sender<()>, recv: Receiver<Float>) -> Self {
        Self {
            ext_sync,
            serial,
            recv,
        }
    }

    pub fn output(&mut self) {
        let Some(mut serial) = self.serial.as_mut() else {
            return;
        };

        // let mut buf = [0; 1];
        // serial.read_exact(&mut buf);

        loop {
            // self.ext_sync.send(()).unwrap();
            let sample = self.recv.recv().unwrap();

            let mut sample_bytes: Vec<u8> = ((sample as f64 * i32::MAX as f64) as i32)
                .to_be_bytes()
                .into_iter()
                .collect();

            // serial.read_exact(&mut buf);

            serial.write(&sample_bytes[..4]);
            // info!("serial sent to hardware");
        }
    }
}

pub struct Output {
    /// used for internal synchronization with the audio buffer sent to rodio
    int_sync: Sender<Float>,
    hw_send: Sender<Float>,
    /// the current sample
    sample: Float,
    /// the rodio output stream, it isn't used but must never be dropped else audio output will cease
    _stream: OutputStream,
    pub volume: Float,
    // hw_audio: HWAudio,
    // hw_audio_thread: JoinHandle<()>,
}

impl Output {
    pub fn new(
        ext_sync: Sender<()>,
    ) -> (
        Self,
        (
            OutputStreamHandle,
            impl Source<Item = f32> + Iterator<Item = f32>,
        ),
    ) {
        info!("making audio output struct");
        let sample = 0.0;
        let (int_sync, rx) = unbounded();
        ext_sync.send(()).unwrap();
        let audio = Audio::new(ext_sync.clone(), rx);
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let (hw_send, hw_rx) = bounded(1);

        #[cfg(feature = "hardware")]
        let hw_audio = {
            let serial: Option<TTYPort> = Some(
                serialport::new("/dev/ttyACM0", 921600)
                    .timeout(Duration::from_nanos(
                        // (1.0 / SAMPLE_RATE as f64 * 10000.0) as u64,
                        1,
                    ))
                    .open_native()
                    .unwrap(),
            );

            let mut audio = HWAudio::new(serial, ext_sync, hw_rx);

            // spawn(move || audio.output())
            audio
        };

        #[cfg(not(feature = "hardware"))]
        let hw_audio_thread = {
            let serial: Option<TTYPort> = None;

            let mut audio = HWAudio::new(serial, ext_sync, hw_rx);

            // spawn(move || audio.output())
        };
        // info!("playing audio struct");
        info!("returning audio struct");

        (
            Self {
                // audio,
                // ext_sync,
                int_sync,
                sample,
                _stream,
                volume: 1.0,
                // hw_audio_thread,
                hw_send,
            },
            // spawn(async move {
            //     stream_handle.play_raw(audio).unwrap();
            // }),
            #[cfg(feature = "hardware")]
            (stream_handle, hw_audio),
            #[cfg(not(feature = "hardware"))]
            (stream_handle, audio),
        )
    }

    pub fn set_volume(&mut self, volume: Float) {
        self.volume = volume;
    }
}

impl Module for Output {
    fn get_samples(&mut self) -> Vec<(u8, Float)> {
        vec![(0, self.sample)]
    }

    fn recv_samples(&mut self, _input_n: u8, samples: &[Float]) {
        let sample: Float = samples.iter().sum();
        self.sample = (sample * self.volume).tanh();
        // warn!("sample -> {sample}");

        // if let Err(e) = self.int_sync.send(self.sample) {
        //     error!("could not send sample to Audio struct. got error: {e}");
        // };

        #[cfg(not(feature = "hardware"))]
        if let Err(e) = self.int_sync.send(self.sample) {
            error!("could not send sample to Audio struct. got error: {e}");
        };

        #[cfg(feature = "hardware")]
        if let Err(e) = self.hw_send.send(self.sample) {
            error!("could not send sample to hardware audio controller struct. got error: {e}");
        }
    }

    fn get_input_names() -> impl Iterator<Item = impl std::fmt::Display> {
        ["Audio In"].iter()
    }

    fn get_output_names() -> impl Iterator<Item = impl std::fmt::Display> {
        ["Audio Out"].iter()
    }
}

unsafe impl Sync for Output {}
unsafe impl Send for Output {}
