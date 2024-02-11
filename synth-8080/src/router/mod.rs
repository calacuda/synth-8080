use crate::{
    common::{Module, ModuleType},
    Float,
};
use log::warn;
use tracing::*;

// pub type NConnections = u8;
// // one sync per module
// pub type ModuleIns = (Arc<[ModuleIn]>, (Sender<()>, Receiver<()>));
// pub type AllInputs = Arc<[ModuleIns]>;

// pub type AdminModuleIns = Arc<&[AdminModuleIn]>;
// pub type AllAdminInputs = Arc<&[AdminModuleIns]>;

// pub type Router = Arc<(AllInputs, AllAdminInputs)>;
// pub type Router = Arc<(AllInputs, AllInputs)>;
// pub type Router = Arc<Inputs>;
//
// #[derive(Clone, Debug)]
// pub struct Inputs {
//     pub in_s: AllInputs,
//     pub admin_in_s: AllInputs,
//     pub sync: Receiver<()>,
// }
//
// pub trait RoutingTable {
//     /// used to make the connection described by the `connection` param.
//     fn inc_connect_counter(&self, connection: Connection);
//     /// used to un-make the connection described by the `connection` param.
//     fn dec_connect_counter(&self, connection: Connection);
// }
//
// impl RoutingTable for Router {
//     fn inc_connect_counter(&self, connection: Connection) {
//         // increment active_connection counter
//         let mut counter = self.in_s[connection.dest_module as usize].0
//             [connection.dest_input as usize]
//             .active_connections
//             .lock()
//             .unwrap();
//         *counter += 1;
//         // info!(
//         //     "incremented the active connection counter ({counter}) for connection: {}:{} => {}:{}, is admin: {}",
//         //     connection.src_module, connection.src_output, connection.dest_module, connection.dest_input, connection.src_admin
//         // );
//     }
//
//     fn dec_connect_counter(&self, connection: Connection) {
//         // decrement active_connections counter
//         let mut active_cons = self.in_s[connection.dest_module as usize].0
//             [connection.dest_input as usize]
//             .active_connections
//             .lock()
//             .unwrap();
//
//         *active_cons -= 1;
//         // info!("active connections after decrement: {active_cons}");
//     }
// }
//
// #[derive(Clone, Debug)]
// pub struct ModuleInRX {
//     pub recv: Receiver<Float>,
//     pub send: Sender<()>,
// }
//
// #[derive(Clone, Debug)]
// pub struct ModuleInTX {
//     pub recv: Receiver<()>,
//     pub send: Sender<Float>,
// }
//
// #[derive(Debug, Clone)]
// pub struct ModuleIn {
//     pub active_connections: Arc<Mutex<NConnections>>,
//     pub sample: Arc<Mutex<Float>>,
//     pub tx_rx: (Sender<Float>, Receiver<Float>),
// }
//
// impl ModuleIn {
//     pub fn new() -> Self {
//         // change to bounded(0) if there are messaging problems or latency/syncronization issues
//         // let (tx, rx): (Sender<Float>, Receiver<Float>) = unbounded();
//         let tx_rx: (Sender<Float>, Receiver<Float>) = unbounded();
//
//         ModuleIn {
//             active_connections: Arc::new(Mutex::new(0)),
//             sample: Arc::new(Mutex::new(0.0)),
//             tx_rx,
//         }
//     }
// }
//
// #[derive(Debug, Clone)]
// pub struct AdminModuleIn {
//     pub active_connections: Arc<Mutex<NConnections>>,
//     pub input: ModuleInTX,
//     pub output: ModuleInRX,
// }
//
// impl AdminModuleIn {
//     pub fn new() -> Self {
//         // change to bounded(0) if there are messaging problems or latency/syncronization issues
//         let (i_tx_i, i_rx_i): (Sender<Float>, Receiver<Float>) = unbounded();
//         let (i_tx_o, i_rx_o): (Sender<()>, Receiver<()>) = unbounded();
//
//         AdminModuleIn {
//             active_connections: Arc::new(Mutex::new(0)),
//             output: ModuleInRX {
//                 recv: i_rx_i,
//                 send: i_tx_o,
//             },
//             input: ModuleInTX {
//                 recv: i_rx_o,
//                 send: i_tx_i,
//             },
//         }
//     }
// }

#[derive(Default)]
pub struct Modules {
    // /// the SINGULAR output module
    // pub output: crate::output::Output,
    /// a list of the echo modules
    pub echo: Vec<crate::echo::Echo>,
    /// a list of the LFOs
    pub lfo: Vec<crate::lfo::Lfo>,
    /// a list of the VCOs
    pub vco: Vec<crate::vco::Vco>,
    /// a list of evnvelope filters
    pub filter: Vec<crate::envelope::EnvelopeFilter>,
    // pub reverb: Vec<(Vec<Input>, Vec<Output>)>,
    // pub mid_pass: Vec<(Vec<Input>, Vec<Output>)>,
    // pub gain: Vec<(Vec<Input>, Vec<Output>)>,
    // pub delay: Vec<(Vec<Input>, Vec<Output>)>,
    // pub chorus: Vec<(Vec<Input>, Vec<Output>)>,
    // pub audio_in: Vec<(Vec<Input>, Vec<Output>)>,
    /// allows for easier indexing into this struct. the index of the items in this Vec corespond
    /// to the modules ID
    pub indeces: Vec<(ModuleType, usize)>,
}

impl Modules {
    pub async fn get_output(&mut self, id: usize) -> Option<Vec<(u8, Float)>> {
        if id == 0 {
            return Some(Vec::new());
        }

        let (mod_type, i) = self.indeces.get(id - 1)?;
        // info!("({mod_type:?}, {i})");
        // info!("n vcos {}", self.vco.len());

        Some(match mod_type {
            ModuleType::Vco => self.vco[*i].get_samples().await,
            ModuleType::Lfo => self.lfo[*i].get_samples().await,
            ModuleType::EnvFilter => self.filter[*i].get_samples().await,
            ModuleType::Echo => self.echo[*i].get_samples().await,
            _ => {
                error!("{mod_type:?} is not yet in Modules.get_output(...)'s match statement. pls fix that");
                return None;
            }
        })
    }

    pub async fn send_sample_to(&mut self, id: usize, input: usize, samples: &[Float]) {
        if id == 0 {
            warn!("break");
            // self.output.recv_samples(0, samples);
            return;
        }

        let (mod_type, i) = self.indeces[id - 1];

        match mod_type {
            ModuleType::Vco => self.vco[i].recv_samples(input as u8, samples).await,
            ModuleType::Lfo => self.lfo[i].recv_samples(input as u8, samples).await,
            ModuleType::EnvFilter => self.filter[i].recv_samples(input as u8, samples).await,
            ModuleType::Echo => self.echo[i].recv_samples(input as u8, samples).await,
            _ => {
                error!("{mod_type:?} is not yet in Modules.get_output(...)'s match statement. pls fix that");
                return;
            }
        }
    }
}

// impl FromIterator<ModuleType> for Modules {
//     fn from_iter<I: IntoIterator<Item = ModuleType>>(iter: I) -> Self {
impl From<&[ModuleType]> for Modules {
    fn from(iter: &[ModuleType]) -> Self {
        let mut s = Self::default();

        iter.into_iter().for_each(|mod_type| match mod_type {
            ModuleType::Vco => {
                s.vco.push(crate::vco::Vco::new((s.indeces.len()) as u8));
                s.indeces.push((*mod_type, s.vco.len() - 1));
            }
            ModuleType::Lfo => {
                s.lfo.push(crate::lfo::Lfo::new((s.indeces.len()) as u8));
                s.indeces.push((*mod_type, s.lfo.len() - 1));
            }
            ModuleType::EnvFilter => {
                s.filter.push(crate::envelope::EnvelopeFilter::new(
                    (s.indeces.len() - 1) as u8,
                ));
                s.indeces.push((*mod_type, s.filter.len() - 1));
            }
            ModuleType::Echo => {
                s.echo.push(crate::echo::Echo::new((s.indeces.len()) as u8));
                s.indeces.push((*mod_type, s.echo.len() - 1));
            }
            _ => {
                error!(
                    "{mod_type:?} is not yet in Modules.from(...)'s match statement. pls fix that"
                );
            }
        });

        s
    }
}

// // pub fn router_send_sample<'a>(router: Router, con: Connection, value: Float) -> anyhow::Result<()> {
// pub fn router_send_sample(router: Router, con: Connection, value: Float) -> anyhow::Result<()> {
//     // while let Err(e) = router
//     //     .0
//     //     .get(con.dest_module as usize)?
//     //     .get(con.dest_input as usize)?
//     //     .output
//     //     .send
//     //     .send(value)
//     // {
//     //     error!(
//     //         "could not send sample to input: {}, of module: {}. got error: {e}",
//     //         con.dest_input, con.dest_module
//     //     );
//     // }
//
//     // router[con].send.send(value)?;
//     let mut sample = router[con].sample.lock().unwrap();
//     *sample = value;
//
//     Ok(())
// }
//
// pub fn router_read_sample(input: &ModuleIn) -> Float {
//     // loop {
//     // // TODO: consider making this recv ALL samples in the channel (might not be nesseary tho)
//
//     // match input.recv.recv() {
//     //     Ok(sample) => sample,
//     //     Err(e) => {
//     //         error!("failed to recv sample with error: {e}");
//     //         // bail!("{e}");
//     //         0.0
//     //     }
//     // }
//     // // } // .unwrap_or(0.0)
//     // input.recv.recv().into_iter().collect()
//     trace!("router_read_sample");
//     input.sample.lock().unwrap().clone()
//     // let n_cons = *input.active_connections.lock().unwrap();
//     // (0..n_cons).map(|_| input.recv.recv()).collect()
// }
//
// // pub fn router_send_sync(input: &ModuleInRX) {
// //     // info!("sending sync");
// //
// //     while let Err(e) = input.send.send(()) {
// //         error!("coulnd not send sync signal. failed with error {e}");
// //     }
// // }
//
// pub fn router_read_sync(router: Router, _con: Connection) -> anyhow::Result<()> {
//     // let n_cons = {
//     //     let n_cons = router[con].active_connections.lock().unwrap().clone() as usize;
//     //     n_cons
//     // };
//     //
//     // if n_cons == 0 {
//     //     return Ok(());
//     // }
//
//     // for _ in 0..20_000 {
//     //     // if let Ok(_) = router
//     //     //     .0
//     //     //     .get(con.dest_module as usize)
//     //     //     .map_or_else(|| bail!("unkown module {}", con.dest_module), |f| Ok(f))?
//     //     //     .get(con.dest_input as usize)
//     //     //     .map_or_else(
//     //     //         || {
//     //     //             bail!(
//     //     //                 "unkown input: {} on module {}",
//     //     //                 con.dest_input,
//     //     //                 con.dest_module
//     //     //             )
//     //     //         },
//     //     //         |f| Ok(f),
//     //     //     )?
//     //     //     .output
//     //     //     .recv
//     //     //     .recv_timeout(Duration::from_nanos(250))
//     //     // {
//     //     if let Ok(_) = router[con]
//     //         .output
//     //         .recv
//     //         // .recv_timeout(Duration::from_nanos(250))
//     //         .try_recv()
//     //     {
//     //         return Ok(());
//     //         //     error!("failed to read sync with error {e}");
//     //         // } else {
//     //         //     return Ok(());
//     //     }
//     // }
//     router.sync.recv()?;
//
//     Ok(())
//
//     // bail!("could not read sync signal in time");
// }
//
// pub fn mk_module_ins(n: usize) -> Vec<ModuleIn> {
//     (0..n).into_iter().map(|_| ModuleIn::new()).collect()
// }

// pub fn mk_admin_module_ins(n: usize) -> AdminModuleIns {
//     (0..n).into_iter().map(|_| AdminModuleIn::new()).collect()
// }
