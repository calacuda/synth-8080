use crate::{
    common::{notes::Note, Connection, ModuleType},
    output,
    router::Modules,
    JoinHandle,
};
use anyhow::{ensure, Result};
use crossbeam_channel::{unbounded, Receiver};
use std::sync::Mutex;
use tracing::*;

pub mod harware;

#[derive(Clone, Copy, Debug)]
pub struct OscilatorId {
    /// the index of the VCO which will synthesis the note,
    pub vco: usize,
    /// the index of the coresponding filter (i.e the filter coreesponding to `self.vco`)
    pub env: usize,
}

impl OscilatorId {
    pub fn new_s(mod_types: &[ModuleType]) -> Result<Vec<Self>> {
        let mut vco_i_s = Vec::new();
        let mut filter_i_s = Vec::new();

        mod_types
            .iter()
            .enumerate()
            .for_each(|(i, mod_t)| match mod_t {
                ModuleType::Vco => vco_i_s.push(i),
                ModuleType::EnvFilter => filter_i_s.push(i),
                _ => {}
            });

        Ok(vco_i_s
            .into_iter()
            .zip(filter_i_s.into_iter())
            .map(|(vco, env)| Self { vco, env })
            .collect())
    }

    /// returns (vco_id, env_id)
    pub fn get(&self) -> (usize, usize) {
        (self.vco, self.env)
    }
}

pub struct Controller {
    /// the liist of connections
    pub connections: Mutex<Vec<Connection>>,
    pub modules: Mutex<Modules>,
    pub output: Mutex<output::Output>,
    pub sync: Receiver<()>,
    // src_s: HashSet<usize>,
    // dest_s: HashSet<usize>,

    // /// the list of registered modules
    // pub modules: Arc<Mutex<Vec<(ModuleInfo, Box<dyn Module>)>>>,
    // /// a table representing all inputs of all modules
    // pub routing_table: Router,
    // /// a list of join handles for the event loops of all modules
    // pub handles: Arc<Mutex<Vec<JoinHandle>>>,
    // /// list of the locations of the oscilators and coresponding envelope filters
    // pub oscilators: Arc<Mutex<Vec<OscilatorId>>>,
    // /// UART connection to the micro-controller
    // pub serial: Arc<Mutex<Box<dyn SerialPort>>>,
    // pub output: Arc<Output>,
    // TODO: find lib to talk to MIDI device
    // /// Connection to MIDI device
    // pub midi: Arc<Mutex<>>,
}

impl Controller {
    pub async fn new(to_build: &[ModuleType]) -> anyhow::Result<(Self, JoinHandle)> {
        let (tx, sync) = unbounded();
        let (output, jh) = output::Output::new(tx);
        let modules = Mutex::new(Modules::from(to_build));

        Ok((
            Self {
                connections: Mutex::new(Vec::new()),
                modules,
                sync,
                output: Mutex::new(output),
            },
            jh,
        ))
    }

    // pub fn step(&mut self) {
    //     // loop {
    //     // warn!("foobar");
    //     if let Err(e) = self.sync.recv() {
    //         error!("error recieving sync message: {e}");
    //     };
    //
    //     let mut src_samples = [[0.0; 16]; u8::MAX as usize];
    //
    //     for src in 0..u8::MAX as usize {
    //         if let Some(mods) = self.modules.lock().unwrap().get_output(src) {
    //             // println!("foobar");
    //             // warn!("mod_type {} => {:?}", src, self.modules.indeces.get(src));
    //             mods.into_iter()
    //                 .for_each(|(output, sample)| src_samples[src][output as usize] += sample);
    //         } else {
    //             break;
    //         }
    //     }
    //
    //     let mut dest_samples = [[0.0; 16]; u8::MAX as usize];
    //     let mut destinations: Vec<(u8, u8)> = Vec::with_capacity(256);
    //
    //     for con in self.connections.lock().unwrap().iter() {
    //         dest_samples[con.dest_module as usize][con.dest_input as usize] +=
    //             src_samples[con.src_module as usize][con.src_output as usize];
    //
    //         let dest = (con.dest_module, con.dest_input);
    //
    //         if !destinations.contains(&dest) {
    //             destinations.push(dest);
    //         }
    //     }
    //
    //     for (dest_mod, dest_in) in destinations {
    //         let sample = dest_samples[dest_mod as usize][dest_in as usize];
    //
    //         if dest_mod == 0 {
    //             self.output.recv_samples(0, &vec![sample]);
    //         } else {
    //             self.modules.lock().unwrap().send_sample_to(
    //                 dest_mod as usize,
    //                 dest_in as usize,
    //                 &vec![sample],
    //             );
    //         }
    //     }
    //     // }
    // }

    /// connects src module to dest module
    pub fn connect(
        &self,
        src_module: u8,
        src_output: u8,
        dest_module: u8,
        dest_input: u8,
    ) -> anyhow::Result<()> {
        let con = Connection {
            src_module,
            src_output,
            dest_module,
            dest_input,
        };

        ensure!(
            self.is_connectable(con),
            "the requested connection is not possible"
        );
        ensure!(
            !self.is_connected(con),
            "the requested connection is already made"
        );

        trace!("connecting");

        // self.src_s.insert(src_module as usize);
        self.connections.lock().unwrap().push(con);

        Ok(())
    }

    /// disconnects src module from dest module
    pub fn disconnect(
        &self,
        src_module: u8,
        src_output: u8,
        dest_module: u8,
        dest_input: u8,
    ) -> anyhow::Result<()> {
        let con = Connection {
            src_module,
            src_output,
            dest_module,
            dest_input,
        };

        ensure!(
            self.is_connected(con),
            "the requested connection is possible made, not disconnecting"
        );

        self.connections.lock().unwrap().retain(|c| c != &con);

        Ok(())
    }

    /// disconnects all connections
    pub fn disconnect_all(&self) {
        self.connections.lock().unwrap().clear();
    }

    /// returns `true` if the connection can be made.
    fn is_connectable(&self, _connection: Connection) -> bool {
        // TODO: write this

        // let mods = self.modules;
        // // does src_mod exist
        // let src_mod = mods.get(connection.src_module as usize).is_some();
        // // does src_mod have output
        // let src_out = mods.get(connection.src_output as usize).is_some();
        // // does dest_mod exist
        // let dest_mod = mods.get(connection.dest_module as usize).is_some();
        // // does dest_mod have input
        // let dest_in = mods.get(connection.dest_input as usize).is_some();
        //
        // src_mod && src_out && dest_mod && dest_in
        true
    }

    /// returns `true` if the connection has already been made.
    fn is_connected(&self, connection: Connection) -> bool {
        self.connections.lock().unwrap().contains(&connection)
    }
}

unsafe impl Send for Controller {}
