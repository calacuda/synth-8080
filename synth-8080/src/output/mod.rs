use crate::{
    common::{Connection, Module},
    router::Router,
    spawn, Float, JoinHandle,
};
use anyhow::bail;
use crossbeam_channel::Sender;
use rodio::{OutputStream, Source};
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};
use tracing::*;

// TODO: Add a volume input to output
pub const N_INPUTS: u8 = 1;
pub const N_OUTPUTS: u8 = 0;

#[derive(Clone)]
pub struct Audio {
    router: Router,
    pub inputs: Arc<Mutex<Vec<Connection>>>,
}

impl Audio {
    pub fn new(router: Router, sync: Sender<()>) -> Self {
        let inputs = Arc::new(Mutex::new(Vec::new()));
        let size = router.in_s.len() * 2;
        // (*router.in_s)
        //     .iter()
        //     .flat_map(|a| a.iter())
        //     .collect::<Vec<_>>()
        //     .len();

        (0..size).for_each(|_i| {
            // warn!("initial first i: {i}");

            if let Err(e) = sync.send(()) {
                error!("failed sending sync: {e}");
            }
            // warn!("initial first i: {i}");
        });
        // warn!("router len: {}", router.in_s.len());

        Self { router, inputs }
    }
}

impl Iterator for Audio {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // let input = &self.router.deref().0[self.id][0].input;
        // let ins = &self.router.deref().0[self.id][0];
        // // router_send_sync(input);
        // // // info!("sync sent");
        // // let sample = router_read_sample(input);
        // // // info!("sample => {sample}");
        // let n_cons = ins.active_connections.lock().unwrap();
        // // let sample: Float = (0..(*n_cons) as usize)
        // //     .map(|_i| {
        // //         router_send_sync(&input);
        // //         // info!("reading sample");
        // //         // read sample from connection
        // //         router_read_sample(&input)
        // //     })
        // //     .sum();
        // (0..(*n_cons) as usize).for_each(|_| router_send_sync(&input));
        // let sample: Float = router_read_sample(&input).iter().sum();
        // info!("sample => {sample}");

        // info!(
        //     "sending sync from output module, sending {} sync signals",
        //     self.size
        // );
        // (0..self.size).for_each(|_i| {
        //     // warn!("first i: {i}");
        //     if let Err(e) = self.sync.send(()) {
        //         error!("failed sending sync: {e}");
        //     }
        //
        //     // warn!("second i: {i}");
        // });
        // info!("output module synced succesfully");
        // let sample: Float = samples.sum();

        // let ins = &self.router.deref().in_s[0][0];
        // info!("reading samples");
        // info!(
        //     "recv-ing {} samples",
        //     ins.active_connections.lock().unwrap()
        // );
        // let sample: Float = ins.recv.recv().iter().sum();
        // self.inputs.lock().unwrap().iter().for_each(|con| {
        //     if con.src_admin {
        //         self.router.admin_in_s[con.src_module as usize]
        //             .1
        //              .0
        //             .send(())
        //     } else {
        //         self.router.in_s[con.src_module as usize].1 .0.send(())
        //     };
        // });

        // trace!("Output Module syncing with admin controllers");
        self.router
            .admin_in_s
            .iter()
            .enumerate()
            // .skip(1)
            .for_each(|(_i, (_, (sync_tx, _sync_rx)))| {
                // trace!("admin module : {_i}");
                if let Err(e) = sync_tx.send(()) {
                    error!("Output Module failed to sync with an admin module. got error: {e}");
                }
            });
        // trace!("admin module synced");

        // trace!("Output Module is about to sync with the the other modules");
        self.router
            .in_s
            .iter()
            .enumerate()
            // .skip(1)
            .for_each(|(_i, (_, (sync_tx, _sync_rx)))| {
                // trace!("regular module : {_i}");
                if let Err(e) = sync_tx.send(()) {
                    error!("Output Module failed to sync with another module. got error: {e}");
                }
            });
        // trace!("modules synced");

        // trace!("gather samples");
        let sample: Float = (0..*self.router.in_s[0].0[0].active_connections.lock().unwrap())
            .map(|_| self.router.in_s[0].0[0].tx_rx.1.recv().unwrap_or(0.0))
            // .flatten()
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

pub struct Output {
    pub audio: Audio,
}

impl Output {
    pub fn new(router_table: Router, sync: Sender<()>) -> Self {
        let audio = Audio::new(router_table, sync);
        info!("made audio struct to handle audio out");

        Self { audio }
    }
}

// TODO: make output able to pass its input transparently (so i can visualize audio out)
impl Module for Output {
    fn start(&self) -> anyhow::Result<JoinHandle> {
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
        // trace!("connection => {:?}", connection);
        // if connection.dest_input == 0 {
        //     info!("connecting to output");
        //     self.audio.inputs.lock().unwrap().push(connection);
        // } else {
        //     bail!("invalid input selection");
        // }

        bail!("invalid input selection");

        // Ok(())
    }

    fn disconnect(&self, _connection: Connection) -> anyhow::Result<()> {
        // if connection.dest_input == 0 {
        //     self.audio
        //         .inputs
        //         .lock()
        //         .unwrap()
        //         .retain(|con| *con != connection);
        // } else {
        bail!("invalid input selection");
        // }

        // Ok(())
    }

    // fn n_outputs(&self) -> u8 {
    //     N_OUTPUTS
    // }

    // fn connections(&self) -> std::sync::Arc<std::sync::Mutex<Vec<Connection>>> {
    //     // Arc::new(Mutex::new(Vec::new()))
    //     self.audio.inputs.clone()
    // }
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
