use crate::{common::Connection, Float};
use anyhow::bail;
use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

pub type NConnections = u8;
pub type ModuleIns = Vec<ModuleIn>;
pub type AllInputs = Vec<ModuleIns>;
pub type Router = Arc<AllInputs>;

pub trait RoutingTable {
    /// used to make the connection described by the `connection` param.
    fn inc_connect_counter(&self, connection: Connection);
    /// used to un-make the connection described by the `connection` param.
    fn dec_connect_counter(&self, connection: Connection);
}

impl RoutingTable for Router {
    fn inc_connect_counter(&self, connection: Connection) {
        // increment active_connection counter
        let mut active_cons = self[connection].active_connections.lock().unwrap();
        *active_cons += 1;
    }

    fn dec_connect_counter(&self, connection: Connection) {
        // decrement active_connections counter
        let mut active_cons = self[connection].active_connections.lock().unwrap();
        *active_cons += 1;
    }
}

#[derive(Clone, Debug)]
pub struct ModuleInRX {
    pub recv: Receiver<Float>,
    pub send: Sender<()>,
}

#[derive(Clone, Debug)]
pub struct ModuleInTX {
    pub recv: Receiver<()>,
    pub send: Sender<Float>,
}

#[derive(Debug)]
pub struct ModuleIn {
    pub active_connections: Mutex<NConnections>,
    pub input: ModuleInRX,
    pub output: ModuleInTX,
}

impl ModuleIn {
    pub fn new() -> Self {
        // change to bounded(0) if there are messaging problems or latency/syncronization issues
        let (i_tx_i, i_rx_i): (Sender<Float>, Receiver<Float>) = bounded(0);
        let (i_tx_o, i_rx_o): (Sender<()>, Receiver<()>) = bounded(0);

        ModuleIn {
            active_connections: Mutex::new(0),
            input: ModuleInRX {
                recv: i_rx_i,
                send: i_tx_o,
            },
            output: ModuleInTX {
                recv: i_rx_o,
                send: i_tx_i,
            },
        }
    }
}

#[derive(Default)]
pub struct Modules {
    // pub adbdr: Vec<crate::adbdr::ADBDR>,
    // pub adsr: Vec<(Vec<Input>, Vec<Output>)>,
    // pub audio_in: Vec<(Vec<Input>, Vec<Output>)>,
    // pub chorus: Vec<(Vec<Input>, Vec<Output>)>,
    // pub delay: Vec<(Vec<Input>, Vec<Output>)>,
    // pub echo: Vec<(Vec<Input>, Vec<Output>)>,
    // pub gain: Vec<(Vec<Input>, Vec<Output>)>,
    // pub lfo: Vec<(Vec<Input>, Vec<Output>)>,
    // pub mid_pass: Vec<(Vec<Input>, Vec<Output>)>,
    pub output: Option<crate::output::Audio>,
    // pub reverb: Vec<(Vec<Input>, Vec<Output>)>,
    pub vco: Vec<crate::vco::Vco>,
}

// impl Index<u8> for Modules {
//     type Output = Box<dyn Module>;
//
//     fn index(&self, index: u8) -> &Self::Output {
//
//     }
// }

pub fn router_send_sample(router: Router, con: Connection, value: Float) -> Option<()> {
    if let Err(e) = router
        .get(con.dest_module as usize)?
        .get(con.dest_input as usize)?
        .output
        .send
        .send(value)
    {
        error!(
            "could not send sample to input: {}, of module: {}. got error: {e}",
            con.dest_input, con.dest_module
        );
    };

    Some(())
}

pub fn router_read_sample(input: &ModuleInRX) -> Float {
    input.recv.recv().unwrap_or(0.0)
}

pub fn router_send_sync(input: &ModuleInRX) {
    // make receiver for this a mutex and send this sync n times where n in the number of active
    // connections
    if let Err(e) = input.send.send(()) {
        error!("coulnd not send sync signal. failed with error {e}");
    }
}

pub fn router_read_sync(router: Router, con: Connection) -> anyhow::Result<()> {
    Ok(router
        .get(con.dest_module as usize)
        .map_or_else(|| bail!("unkown module {}", con.dest_module), |f| Ok(f))?
        .get(con.dest_input as usize)
        .map_or_else(
            || {
                bail!(
                    "unkown input: {} on module {}",
                    con.dest_input,
                    con.dest_module
                )
            },
            |f| Ok(f),
        )?
        .output
        .recv
        .recv()?)
}

pub fn mk_module_ins(n: usize) -> ModuleIns {
    (0..n).into_iter().map(|_| ModuleIn::new()).collect()
}
