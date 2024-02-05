use crate::{
    output,
    router::{
        mk_module_ins, router_read_sample, router_read_sync, router_send_sample, router_send_sync,
        ModuleIn, ModuleIns, Router,
    },
    vco, Float, FLOAT_LEN,
};
use anyhow::{ensure, Result};
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut, Index},
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;
use tracing::{error, info};

pub mod notes;
#[derive(Clone, Copy, Debug)]
pub enum ModuleType {
    Vco,
    Output,
}

impl ModuleType {
    pub async fn builder(&self, routing_table: Router, i: usize) -> Box<dyn Module> {
        let id = i as u8;
        info!("making a {self:?} module");

        let module: Box<dyn Module> = match *self {
            ModuleType::Vco => Box::new(vco::Vco::new(routing_table, id)),
            ModuleType::Output => Box::new(output::Output::new(routing_table, id).await),
        };

        info!("made a {self:?} module");

        module
    }

    pub fn get_info(&self) -> ModuleInfo {
        match self {
            ModuleType::Vco => ModuleInfo {
                n_ins: vco::N_INPUTS,
                n_outs: vco::N_OUTPUTS,
                io: mk_module_ins(vco::N_INPUTS as usize),
            },
            ModuleType::Output => ModuleInfo {
                n_ins: output::N_INPUTS,
                n_outs: output::N_OUTPUTS,
                io: mk_module_ins(output::N_INPUTS as usize),
            },
        }
    }
}

pub struct ModuleInfo {
    pub n_ins: u8,
    pub n_outs: u8,
    pub io: ModuleIns,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Connection {
    pub src_module: u8,
    pub src_output: u8,
    pub dest_module: u8,
    pub dest_input: u8,
}

pub struct IO {
    pub inputs: Vec<(ModuleIn, Box<dyn FnMut(&[Float])>)>,
    pub outputs: Vec<(Arc<Mutex<Vec<Connection>>>, Box<dyn FnMut() -> Float>)>,
}

impl Index<Connection> for Router {
    type Output = ModuleIn;

    fn index(&self, index: Connection) -> &Self::Output {
        &self.deref()[index.dest_module as usize][index.dest_input as usize]
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
    fn start(&self) -> anyhow::Result<JoinHandle<()>>;
    // /// stops the modules event loop
    // fn stop(&self) -> anyhow::Result<()>;
    /// connects the module to another module
    fn connect(&self, connection: Connection) -> anyhow::Result<()>;
    /// disconnects the module from another module
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

fn sync_with_inputs(ins: &mut [(&ModuleIn, Box<dyn FnMut(&[Float])>)]) {
    ins.iter_mut().for_each(|(cons, f)| {
        let n_cons = *cons.active_connections.lock().unwrap();

        let sample: Vec<Float> = (0..n_cons)
            .map(|_i| {
                // send sync signal
                router_send_sync(&cons.input);
                // read sample from connection
                router_read_sample(&cons.input)
            })
            .collect();

        f(&sample);
    })
}

fn send_samples(
    router: Router,
    outs: &mut [(Arc<Mutex<Vec<Connection>>>, Box<dyn FnMut() -> Float>)],
) {
    outs.iter_mut().for_each(|(cons, f)| {
        let sample = f();

        cons.lock().unwrap().iter().for_each(|con| {
            if let Err(e) = router_read_sync(router.clone(), *con) {
                error!("encountered an error waiting for sync message: {e}");
            }
        });

        cons.lock().unwrap().iter().for_each(|connection| {
            router_send_sample(router.clone(), *connection, sample);
        });
    })
}

// TODO: make this a macro
pub fn event_loop(
    router: Router,
    inputs: &mut Vec<(&ModuleIn, Box<dyn FnMut(&[Float])>)>,
    outputs: &mut [(Arc<Mutex<Vec<Connection>>>, Box<dyn FnMut() -> Float>)],
) {
    info!("starting event loop");

    loop {
        sync_with_inputs(inputs);
        send_samples(router.clone(), outputs)
    }
}
