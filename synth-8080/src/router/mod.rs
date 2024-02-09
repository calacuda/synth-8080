use crate::{common::Connection, Float};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::{Arc, Mutex};
use tracing::*;

pub type NConnections = u8;
// one sync per module
pub type ModuleIns = (Arc<[ModuleIn]>, (Sender<()>, Receiver<()>));
pub type AllInputs = Arc<[ModuleIns]>;

// pub type AdminModuleIns = Arc<&[AdminModuleIn]>;
// pub type AllAdminInputs = Arc<&[AdminModuleIns]>;

// pub type Router = Arc<(AllInputs, AllAdminInputs)>;
// pub type Router = Arc<(AllInputs, AllInputs)>;
pub type Router = Arc<Inputs>;

#[derive(Clone, Debug)]
pub struct Inputs {
    pub in_s: AllInputs,
    pub admin_in_s: AllInputs,
    pub sync: Receiver<()>,
}

pub trait RoutingTable {
    /// used to make the connection described by the `connection` param.
    fn inc_connect_counter(&self, connection: Connection);
    /// used to un-make the connection described by the `connection` param.
    fn dec_connect_counter(&self, connection: Connection);
}

impl RoutingTable for Router {
    fn inc_connect_counter(&self, connection: Connection) {
        // increment active_connection counter
        let mut counter = self[connection].active_connections.lock().unwrap();
        *counter += 1;
        info!(
            "incremented the active connection counter ({counter}) for connection: {}:{}, is admin: {}",
            connection.dest_module, connection.dest_input, connection.src_admin
        );
    }

    fn dec_connect_counter(&self, connection: Connection) {
        // decrement active_connections counter
        let mut active_cons = self[connection].active_connections.lock().unwrap();
        *active_cons -= 1;
        // info!("active connections after decrement: {active_cons}");
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

#[derive(Debug, Clone)]
pub struct ModuleIn {
    pub active_connections: Arc<Mutex<NConnections>>,
    pub sample: Arc<Mutex<Float>>,
}

impl ModuleIn {
    pub fn new() -> Self {
        // change to bounded(0) if there are messaging problems or latency/syncronization issues
        // let (tx, rx): (Sender<Float>, Receiver<Float>) = unbounded();

        ModuleIn {
            active_connections: Arc::new(Mutex::new(0)),
            sample: Arc::new(Mutex::new(0.0)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdminModuleIn {
    pub active_connections: Arc<Mutex<NConnections>>,
    pub input: ModuleInTX,
    pub output: ModuleInRX,
}

impl AdminModuleIn {
    pub fn new() -> Self {
        // change to bounded(0) if there are messaging problems or latency/syncronization issues
        let (i_tx_i, i_rx_i): (Sender<Float>, Receiver<Float>) = unbounded();
        let (i_tx_o, i_rx_o): (Sender<()>, Receiver<()>) = unbounded();

        AdminModuleIn {
            active_connections: Arc::new(Mutex::new(0)),
            output: ModuleInRX {
                recv: i_rx_i,
                send: i_tx_o,
            },
            input: ModuleInTX {
                recv: i_rx_o,
                send: i_tx_i,
            },
        }
    }
}

// #[derive(Default)]
// pub struct Modules {
//     // pub adbdr: Vec<crate::adbdr::ADBDR>,
//     // pub adsr: Vec<(Vec<Input>, Vec<Output>)>,
//     // pub audio_in: Vec<(Vec<Input>, Vec<Output>)>,
//     // pub chorus: Vec<(Vec<Input>, Vec<Output>)>,
//     // pub delay: Vec<(Vec<Input>, Vec<Output>)>,
//     // pub echo: Vec<(Vec<Input>, Vec<Output>)>,
//     // pub gain: Vec<(Vec<Input>, Vec<Output>)>,
//     // pub lfo: Vec<(Vec<Input>, Vec<Output>)>,
//     // pub mid_pass: Vec<(Vec<Input>, Vec<Output>)>,
//     pub output: Option<crate::output::Audio>,
//     // pub reverb: Vec<(Vec<Input>, Vec<Output>)>,
//     pub vco: Vec<crate::vco::Vco>,
// }

// impl Index<u8> for Modules {
//     type Output = Box<dyn Module>;
//
//     fn index(&self, index: u8) -> &Self::Output {
//
//     }
// }

// pub fn router_send_sample<'a>(router: Router, con: Connection, value: Float) -> anyhow::Result<()> {
pub fn router_send_sample(router: Router, con: Connection, value: Float) -> anyhow::Result<()> {
    // while let Err(e) = router
    //     .0
    //     .get(con.dest_module as usize)?
    //     .get(con.dest_input as usize)?
    //     .output
    //     .send
    //     .send(value)
    // {
    //     error!(
    //         "could not send sample to input: {}, of module: {}. got error: {e}",
    //         con.dest_input, con.dest_module
    //     );
    // }

    // router[con].send.send(value)?;
    let mut sample = router[con].sample.lock().unwrap();
    *sample = value;

    Ok(())
}

pub fn router_read_sample(input: &ModuleIn) -> Float {
    // loop {
    // // TODO: consider making this recv ALL samples in the channel (might not be nesseary tho)

    // match input.recv.recv() {
    //     Ok(sample) => sample,
    //     Err(e) => {
    //         error!("failed to recv sample with error: {e}");
    //         // bail!("{e}");
    //         0.0
    //     }
    // }
    // // } // .unwrap_or(0.0)
    // input.recv.recv().into_iter().collect()
    trace!("router_read_sample");
    input.sample.lock().unwrap().clone()
    // let n_cons = *input.active_connections.lock().unwrap();
    // (0..n_cons).map(|_| input.recv.recv()).collect()
}

// pub fn router_send_sync(input: &ModuleInRX) {
//     // info!("sending sync");
//
//     while let Err(e) = input.send.send(()) {
//         error!("coulnd not send sync signal. failed with error {e}");
//     }
// }

pub fn router_read_sync(router: Router, con: Connection) -> anyhow::Result<()> {
    // let n_cons = {
    //     let n_cons = router[con].active_connections.lock().unwrap().clone() as usize;
    //     n_cons
    // };
    //
    // if n_cons == 0 {
    //     return Ok(());
    // }

    // for _ in 0..20_000 {
    //     // if let Ok(_) = router
    //     //     .0
    //     //     .get(con.dest_module as usize)
    //     //     .map_or_else(|| bail!("unkown module {}", con.dest_module), |f| Ok(f))?
    //     //     .get(con.dest_input as usize)
    //     //     .map_or_else(
    //     //         || {
    //     //             bail!(
    //     //                 "unkown input: {} on module {}",
    //     //                 con.dest_input,
    //     //                 con.dest_module
    //     //             )
    //     //         },
    //     //         |f| Ok(f),
    //     //     )?
    //     //     .output
    //     //     .recv
    //     //     .recv_timeout(Duration::from_nanos(250))
    //     // {
    //     if let Ok(_) = router[con]
    //         .output
    //         .recv
    //         // .recv_timeout(Duration::from_nanos(250))
    //         .try_recv()
    //     {
    //         return Ok(());
    //         //     error!("failed to read sync with error {e}");
    //         // } else {
    //         //     return Ok(());
    //     }
    // }
    router.sync.recv()?;

    Ok(())

    // bail!("could not read sync signal in time");
}

pub fn mk_module_ins(n: usize) -> Vec<ModuleIn> {
    (0..n).into_iter().map(|_| ModuleIn::new()).collect()
}

// pub fn mk_admin_module_ins(n: usize) -> AdminModuleIns {
//     (0..n).into_iter().map(|_| AdminModuleIn::new()).collect()
// }
