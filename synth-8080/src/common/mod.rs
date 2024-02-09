use crate::{
    echo, envelope, lfo, output,
    router::{mk_module_ins, ModuleIn, Router},
    vco, Float, JoinHandle, FLOAT_LEN,
};
use anyhow::{ensure, Result};
use log::trace;
use std::{
    ops::Index,
    sync::{Arc, Mutex},
};
use tokio::time::{sleep, Duration};
use tracing::*;

pub mod notes;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModuleType {
    Vco,
    Output,
    Lfo,
    Echo,
    EnvFilter,
}

impl ModuleType {
    pub async fn builder(&self, routing_table: Router, i: usize) -> Option<Box<dyn Module>> {
        let id = i as u8;
        info!("making a {self:?} module");

        let module: Option<Box<dyn Module>> = match *self {
            ModuleType::Vco => Some(Box::new(vco::Vco::new(routing_table, id))),
            ModuleType::Output => None,
            ModuleType::Lfo => Some(Box::new(lfo::Lfo::new(routing_table, id))),
            ModuleType::Echo => Some(Box::new(echo::Echo::new(routing_table, id))),
            ModuleType::EnvFilter => {
                Some(Box::new(envelope::EnvelopeFilter::new(routing_table, id)))
            }
        };

        info!("made a {self:?} module");
        // if *self != ModuleType::Output {
        //     sleep(Duration::from_secs(1)).await;
        // }

        module
    }

    pub fn get_info(&self) -> (ModuleInfo, ModuleInfo) {
        match self {
            ModuleType::Vco => (
                ModuleInfo {
                    n_ins: vco::N_INPUTS,
                    n_outs: vco::N_OUTPUTS,
                    io: mk_module_ins(vco::N_INPUTS as usize),
                    mod_type: *self,
                },
                ModuleInfo {
                    n_ins: vco::N_OUTPUTS,
                    n_outs: vco::N_INPUTS,
                    io: mk_module_ins(vco::N_OUTPUTS as usize),
                    mod_type: *self,
                },
            ),
            ModuleType::Output => (
                ModuleInfo {
                    n_ins: output::N_INPUTS,
                    n_outs: output::N_OUTPUTS,
                    io: mk_module_ins(output::N_INPUTS as usize),
                    mod_type: *self,
                },
                ModuleInfo {
                    n_ins: output::N_OUTPUTS,
                    n_outs: output::N_INPUTS,
                    io: mk_module_ins(output::N_OUTPUTS as usize),
                    mod_type: *self,
                },
            ),
            ModuleType::Lfo => (
                ModuleInfo {
                    n_ins: lfo::N_INPUTS,
                    n_outs: lfo::N_OUTPUTS,
                    io: mk_module_ins(lfo::N_INPUTS as usize),
                    mod_type: *self,
                },
                ModuleInfo {
                    n_ins: lfo::N_OUTPUTS,
                    n_outs: lfo::N_INPUTS,
                    io: mk_module_ins(lfo::N_OUTPUTS as usize),
                    mod_type: *self,
                },
            ),
            ModuleType::Echo => (
                ModuleInfo {
                    n_ins: echo::N_INPUTS,
                    n_outs: echo::N_OUTPUTS,
                    io: mk_module_ins(echo::N_INPUTS as usize),
                    mod_type: *self,
                },
                ModuleInfo {
                    n_ins: echo::N_OUTPUTS,
                    n_outs: echo::N_INPUTS,
                    io: mk_module_ins(echo::N_OUTPUTS as usize),
                    mod_type: *self,
                },
            ),
            ModuleType::EnvFilter => (
                ModuleInfo {
                    n_ins: envelope::N_INPUTS,
                    n_outs: envelope::N_OUTPUTS,
                    io: mk_module_ins(envelope::N_INPUTS as usize),
                    mod_type: *self,
                },
                ModuleInfo {
                    n_ins: envelope::N_OUTPUTS,
                    n_outs: envelope::N_INPUTS,
                    io: mk_module_ins(envelope::N_OUTPUTS as usize),
                    mod_type: *self,
                },
            ),
        }
    }
}

pub struct ModuleInfo {
    pub n_ins: u8,
    pub n_outs: u8,
    pub io: Vec<ModuleIn>,
    pub mod_type: ModuleType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Connection {
    pub src_module: u8,
    pub src_output: u8,
    pub dest_module: u8,
    pub dest_input: u8,
    pub src_admin: bool,
    pub dest_admin: bool,
}

// pub struct IO {
//     pub inputs: Vec<(ModuleIn, Box<dyn FnMut(&[Float])>)>,
//     pub outputs: Vec<(Arc<Mutex<Vec<Connection>>>, Box<dyn FnMut() -> Float>)>,
// }

impl Index<Connection> for Router {
    type Output = ModuleIn;

    fn index(&self, index: Connection) -> &Self::Output {
        // if !index.src_admin {
        //     &self.in_s[index.src_module as usize].0[index.src_output as usize]
        // } else {
        //     &self.admin_in_s[index.src_module as usize].0[index.src_output as usize]
        // }
        &self.in_s[index.src_module as usize].0[index.src_output as usize]
    }
}

// impl IndexMut<Connection> for Router {
//     // type Output = Option<Float>;
//
//     fn index_mut(&mut self, index: Connection) -> &mut Self::Output {
//         &mut self[index]
//     }
// }

pub trait Module {
    /// start the modules evvent loop
    fn start(&self) -> anyhow::Result<JoinHandle>;
    /// connects the module to another module
    // fn connect(&self, connection: Connection) -> anyhow::Result<()> {
    //     ensure!(
    //         connection.src_output < self.n_outputs(),
    //         "invalid output selection"
    //     );
    //     ensure!(
    //         !self.connections().lock().unwrap().contains(&connection),
    //         "module already connected"
    //     );
    //     self.connections().lock().unwrap().push(connection);
    //
    //     Ok(())
    // }
    // /// returns how many outputs the module has
    // fn n_outputs(&self) -> u8;
    // /// returns the Arc<Mutex<Vec<Connection>>> that stores the currently connected connections
    // fn connections(&self) -> Arc<Mutex<Vec<Connection>>>;
    /// disconnects the module from another module
    // fn disconnect(&self, connection: Connection) -> anyhow::Result<()> {
    //     ensure!(
    //         connection.src_output < self.n_outputs(),
    //         "invalid output selection"
    //     );
    //     ensure!(
    //         self.connections().lock().unwrap().contains(&connection),
    //         "module not connected"
    //     );
    //     self.connections()
    //         .lock()
    //         .unwrap()
    //         .retain(|out| *out != connection);
    //
    //     Ok(())
    // }
    fn connect(&self, connection: Connection) -> anyhow::Result<()>;
    fn disconnect(&self, connection: Connection) -> anyhow::Result<()>;
}

pub fn mk_float(b: &[u8]) -> Result<Float> {
    ensure!(b.len() == FLOAT_LEN, "length of bytes bust be ");

    Ok(Float::from_le_bytes([
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
    ]))
}

pub fn bend_range() -> Float {
    (2.0 as Float).powf(2.0 / 12.0)
}

pub async fn sync_with_inputs(
    // id: usize,
    router: &Router,
    ins: &mut Vec<(
        Arc<Mutex<Vec<Connection>>>,
        Box<dyn FnMut(Vec<Float>) + Send>,
    )>,
) {
    ins.iter_mut()
        .enumerate()
        .for_each(|(_i, (connections, f))| {
            // let n_cons = *mods.active_connections.lock().unwrap();
            // // info!("{n_cons} active connections");
            //
            // if n_cons > 0 {
            // info!("syncing with input");
            // let samples = (0..n_cons)
            //     .map(|_| {
            //         // router_send_sync(&cons));
            //                             // info!("foo");
            //     })
            //     .collect();
            // warn!("n connections: {}", connections.lock().unwrap().len());

            // trace!("syncing with input : {i}");
            let samples: Vec<Float> = connections
                .lock()
                .unwrap()
                .iter()
                .map(|con| router[*con].sample.lock().unwrap().clone())
                .collect();
            // trace!("syncd with input : {i}");

            // warn!("samples: {:?}", samples);
            if !samples.is_empty() {
                f(samples);
            }
            // info!("read samples from inputs");
            // }
        });
}

pub async fn send_samples(
    router: &Router,
    id: usize,
    outs: &mut [(usize, Box<dyn FnMut() -> Float + Send>)],
) {
    outs.iter_mut().for_each(|(out, f)| {
        let sample = f();
        // info!("sample {sample}");

        // cons.lock().unwrap().iter().for_each(|con| {
        // info!("about to read sync");

        // if let Err(e) = router_read_sync(router.clone(), *con) {
        //     error!("encountered an error waiting for sync message: {e}");
        // }

        // debug!("sending samples from {id}:{out}");

        // if con.src_admin {
        //     info!("sending from the controller {sample}");
        // }
        let mut bucket = router.in_s[id].0[*out].sample.lock().unwrap();
        *bucket = sample;
        // if let Err(e) = router_send_sample(router.clone(), *con, sample) {
        //     error!("encountered an error sending sample: {e}");
        // };
        // });
    });

    trace!("exiting send_samples");
}

pub async fn event_loop<'a>(
    router: Router,
    // sync: Receiver<()>,
    mut inputs: Vec<(
        Arc<Mutex<Vec<Connection>>>,
        Box<dyn FnMut(Vec<Float>) + Send>,
    )>,
    mut outputs: (usize, Vec<(usize, Box<dyn FnMut() -> Float + Send>)>),
) {
    info!("starting event loop");

    if let Err(e) = router.sync.recv() {
        error!("failed to wait for sync from output module, {e}");
    }

    let id = outputs.0;

    trace!("entering indefinate loop inside the event_loop fucntion for module {id}");

    loop {
        // TODO: check graceful shutdown channel and quit if should.
        // info!("will attempt to sync with inputs");

        // if let Err(e) = router.sync.recv() {
        //     error!("failed to wait for sync from output module, {e}");
        // }

        // if let Err(e) = router.in_s[outputs.0].1 .1.recv() {
        //     error!(
        //         "sync recv failed for module: {}. sync failed with error: {e}",
        //         outputs.0
        //     );
        // }

        // sync_with_inputs(&router, &mut inputs).await;
        // // trace!("synced with inputs");
        // send_samples(&router, outputs.0, &mut outputs.1).await;
        // // trace!("samples sent");

        // sync with inputs
        inputs
            .iter_mut()
            .enumerate()
            .for_each(|(_i, (connections, f))| {
                // trace!("syncing with input : {i}");
                let samples: Vec<Float> = connections
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|con| router[*con].sample.lock().unwrap().clone())
                    .collect();
                // trace!("syncd with input : {i}");

                // debug!("samples: {:?}", samples);
                if !samples.is_empty() {
                    f(samples);
                }
            });

        // gererate samples for output

        // let mut output_samples =
        outputs.1.iter_mut().for_each(|(out, f)| {
            // let sample = f();
            let mut bucket = router.in_s[id].0[*out].sample.lock().unwrap();
            // (bucket, f())
            // debug!("sending samples from {id}:{out}");
            *bucket = f();

            // let mut bucket = router.in_s[id].0[*out].sample.lock().unwrap();
            // *bucket = sample;
        });

        if let Err(e) = router.in_s[outputs.0].1 .1.recv() {
            error!(
                "sync recv failed for module: {}. sync failed with error: {e}",
                outputs.0
            );
        }

        // upload samples
        // output_samples.for_each(|(mut bucket, sample)| {
        //     // debug!("sending samples from {id}:{out}");
        //
        //     // let mut bucket = router.in_s[id].0[*out].sample.lock().unwrap();
        //     *bucket = sample;
        // });

        // sleep(Duration::from_nanos(1)).await;
    }
}

pub async fn admin_event_loop<'a>(
    router: Router,
    // sync: Receiver<()>,
    // mut inputs: Vec<(&'a ModuleIn, Box<dyn FnMut(Vec<Float>) + Send>)>,
    mut outputs: (usize, Vec<(usize, Box<dyn FnMut() -> Float + Send>)>),
) {
    if let Err(e) = router.sync.recv() {
        error!("[admin] failed to wait for sync from output module, {e}");
    }

    loop {
        // TODO: check graceful shutdown channel and quit if should.
        // if let Err(e) = sync.recv() {
        //     error!("failed to wait for sync from output module, {e}");
        // }

        // info!("[admin] id => {}", outputs.0);

        outputs.1.iter_mut().for_each(|(out, f)| {
            // let sample = f();
            let mut bucket = router.admin_in_s[outputs.0].0[*out].sample.lock().unwrap();
            *bucket = f();
        });

        if let Err(e) = router.admin_in_s[outputs.0].1 .1.recv() {
            error!(
                "[admin] sync recv failed for admin module: {}. sync failed with error: {e}",
                outputs.0
            );
        }

        // send_samples(router.clone(), outputs.0, &mut outputs.1);

        // info!("will attempt to sync with inputs");
        // sync_with_inputs(&mut inputs);
        // sleep(Duration::from_nanos(1)).await;
    }
}
