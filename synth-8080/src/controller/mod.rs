use crate::{
    adbdr,
    common::{event_loop, notes::Note, Connection, Module, ModuleInfo, ModuleType},
    router::{router_send_sample, AllInputs, ModuleIn, Router, RoutingTable},
    vco, Float,
};
use anyhow::{bail, ensure, Result};
use futures::future::join_all;
use serialport::SerialPort;
use std::{
    collections::HashMap,
    io,
    ops::{Deref, DerefMut, Index},
    sync::{Arc, Mutex},
};
use tokio::{
    spawn,
    task::JoinHandle,
    time::{sleep, Duration},
};
use tracing::{error, info, trace, warn};

#[derive(Clone, Copy, Debug)]
pub enum EnvelopeType {
    ADBDR,
    // *** Not yet programmed *** //

    // AHDSR,
    // ADSR,
    // AD, // maybe NOPE (replaced with adr to stop that anoying pop sound)
    // AR,
    // ADR,
    // ADS, // NOPE
}

#[derive(Clone, Copy, Debug)]
pub struct EnvelopeFilters {
    /// the index of the adbdr envelope filter
    pub adbdr: usize,
    // *** Not yet programmed *** //

    // /// the index of the ahdsr envelope filter
    // pub ahdsr: usize,
    // /// the index of the adsr envelope filter
    // pub adsr: usize,
    // /// the index of the ad envelope filter
    // pub ad: usize, // maybe NOPE (replaced with adr)
    // /// the index of the ar envelope filter
    // pub ar: usize,
    // /// the index of the adr envelope filter
    // pub adr: usize,
    // /// the index of the ads envelope filter
    // pub ads: usize, // NOPE
}

impl EnvelopeFilters {
    pub fn new(i: usize, mod_type_map: &HashMap<ModuleType, Vec<usize>>) -> Result<Self> {
        Ok(Self {
            adbdr: mod_type_map
                .get(&ModuleType::Adbdr)
                .map_or_else(|| bail!("there were fewer ADBDR filters then VCO_s this is not currently supported."), |val| Ok(val[i]))?,
        })
    }

    pub fn get(&self, mod_type: EnvelopeType) -> usize {
        self[mod_type]
    }
}

impl Index<EnvelopeType> for EnvelopeFilters {
    type Output = usize;

    fn index(&self, index: EnvelopeType) -> &Self::Output {
        match index {
            EnvelopeType::ADBDR => &self.adbdr,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OscilatorId {
    /// the index of the VCO which will synthesis the note,
    pub vco: usize,
    /// a list of the differnet envelope filters that corespond to `self.vco`
    pub envelope: EnvelopeFilters,
}

impl OscilatorId {
    pub fn new(
        vco_id: usize,
        i: usize,
        mod_type_map: &HashMap<ModuleType, Vec<usize>>,
    ) -> Result<Self> {
        Ok(Self {
            vco: vco_id,
            envelope: EnvelopeFilters::new(i, &mod_type_map)?,
        })
    }

    pub fn new_s(mod_types: &[ModuleType]) -> Result<Vec<Self>> {
        let env_type = [ModuleType::Vco, ModuleType::Adbdr];
        let mut mod_type_map: HashMap<ModuleType, Vec<usize>> = HashMap::new();

        env_type.iter().for_each(|mt| {
            mod_type_map.insert(
                *mt,
                mod_types
                    .iter()
                    .enumerate()
                    .filter_map(|(i, mod_type)| if mod_type == mt { Some(i) } else { None })
                    .collect(),
            );
        });

        mod_type_map
            .get(&ModuleType::Vco)
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, vco_id)| Self::new(*vco_id, i, &mod_type_map))
            .collect()
    }

    /// returns (vco_id, env_id)
    pub fn get(&self, env_type: EnvelopeType) -> (usize, usize) {
        (self.vco, self.envelope.get(env_type))
    }
}

pub struct Controller {
    /// the liist of connections
    pub connections: Arc<Mutex<Vec<Connection>>>,
    /// the list of registered modules
    // TODO: also store in a struc that sore the modules by type
    pub modules: Arc<Mutex<Vec<(ModuleInfo, Box<dyn Module>)>>>,
    /// a table representing all inputs of all modules
    pub routing_table: Router,
    /// a list of join handles for the event loops of all modules
    pub handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    /// list of the locations of the oscilators and coresponding envelope filters
    pub oscilators: Arc<Mutex<Vec<OscilatorId>>>,
    /// which envelope filter is active
    pub envelope_type: Arc<Mutex<EnvelopeType>>,
    /// UART connection to the micro-controller
    pub serial: Arc<Mutex<Box<dyn SerialPort>>>,
    // TODO: find lib to talk to MIDI device
    // /// Connection to MIDI device
    // pub midi: Arc<Mutex<>>,
}

impl Controller {
    pub async fn new(to_build: &[ModuleType]) -> anyhow::Result<Self> {
        let connections = Vec::new();
        let info = to_build.iter().map(|mod_type| mod_type.get_info());

        let normal_name_space: AllInputs = info.clone().map(|mod_type| mod_type.0.io).collect();
        let admin_name_space: AllInputs = info.clone().map(|mod_type| mod_type.0.io).collect();
        let routing_table: Router = Arc::new((normal_name_space, admin_name_space));
        info!("made routing table");
        // make routing_table

        let modules: Vec<(ModuleInfo, Box<dyn Module>)> = join_all(
            to_build
                .iter()
                // .zip(info)
                .enumerate()
                .map(|(i, mod_type)| mod_type.builder(routing_table.clone(), i)),
        )
        .await
        .into_iter()
        .zip(info)
        .map(|(m, i)| (i.0, m))
        .collect();
        info!("made module list");
        // make modules

        // start modules
        let handles = Arc::new(Mutex::new(
            modules
                .iter()
                .map(|(_info, module)| module.start())
                .collect::<anyhow::Result<Vec<JoinHandle<()>>>>()?,
        ));
        info!("started the modules");

        let serial = Arc::new(Mutex::new(serialport::new("/dev/ttyACM0", 115200).open()?));
        let envelope_type = Arc::new(Mutex::new(EnvelopeType::ADBDR));
        let oscilators = Arc::new(Mutex::new(OscilatorId::new_s(to_build)?));

        Ok(Self {
            connections: Arc::new(Mutex::new(connections)),
            modules: Arc::new(Mutex::new(modules)),
            routing_table,
            handles,
            oscilators,
            envelope_type,
            serial,
        })
    }

    /// starts an event loop to listen for events over both serial and MIDI.
    pub fn start(&self) -> JoinHandle<()> {
        // TODO: trun LED red

        let port = self.serial.clone();
        let osc = self.oscilators.clone();
        let env_type = self.envelope_type.clone();
        let router = self.routing_table.clone();
        let connections = self.connections.clone();
        let handles = self.handles.clone();
        // handle serial events from micro controller
        // sleep(Duration::from_secs(1)).await
        let mut serial_buf: Vec<u8> = vec![0; 1000];
        let osc_sample = Arc::new(Mutex::new(0.0));
        let env_sample = Arc::new(Mutex::new(0.0));
        let note = osc_sample.clone();
        let filter_open = env_sample.clone();

        let (osc_id, env_id) = osc.lock().unwrap()[0].get(*env_type.lock().unwrap());

        let gen_env_sample: Box<dyn FnMut() -> Float + Send> = Box::new(move || {
            let sample = *env_sample.lock().unwrap();
            // info!("envelope filter is open: {}", sample >= 0.75);

            sample
        });
        let jh = Controller::spawn_admin_cmd(
            gen_env_sample,
            env_id as u8,
            adbdr::ENVELOPE_IN,
            router.clone(),
            connections.clone(),
        );
        handles.lock().unwrap().push(jh);

        let gen_osc_sample: Box<dyn FnMut() -> Float + Send> = Box::new(move || {
            // info!("about to send pitch");
            let sample = osc_sample.lock().unwrap().clone();
            // info!("setting vco pitch to {sample}");

            sample
        });
        let jh = Controller::spawn_admin_cmd(
            gen_osc_sample,
            osc_id as u8,
            vco::PITCH_INPUT,
            router.clone(),
            connections.clone(),
        );
        handles.lock().unwrap().push(jh);

        spawn(async move {
            loop {
                match port.lock().unwrap().read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        let raw_input = String::from_utf8_lossy(&serial_buf[..t]);
                        let cmd = raw_input.trim();

                        info!("recieved command: {cmd:?}");

                        if cmd == "play" {
                            // info!("setting Notes");
                            let mut os = note.lock().unwrap();
                            *os = Note::A4.into();

                            let mut es = filter_open.lock().unwrap();
                            *es = 1.0;
                        } else if cmd == "stop" {
                            let mut os = note.lock().unwrap();
                            *os = 0.0;

                            let mut es = filter_open.lock().unwrap();
                            *es = 0.0;
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        })
    }

    /// sends a samples to destnation module and input
    pub fn spawn_admin_cmd(
        gen_sample: Box<dyn FnMut() -> Float + Send>,
        dest_module: u8,
        dest_input: u8,
        router: Router,
        connections: Arc<Mutex<Vec<Connection>>>,
    ) -> JoinHandle<()> {
        let con = Connection {
            src_module: dest_module,
            src_output: dest_input,
            dest_module,
            dest_input,
            src_admin: true,
            dest_admin: false,
        };

        spawn(async move {
            let ins: &Vec<ModuleIn> = (*router)
                .1
                .get(dest_module as usize)
                .expect("this VCO Module was not found in the routing table struct.")
                .as_ref();
            let outputs = vec![(Arc::new(Mutex::new(vec![con])), gen_sample)];

            let do_nothing: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |_samples: Vec<Float>| warn!("doing nothing"));
            let inputs = vec![(&ins[dest_input as usize], do_nothing)];

            connections.lock().unwrap().push(con);
            router.inc_connect_counter(con);
            event_loop(router.clone(), inputs, outputs).await;
        })
    }

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
            src_admin: false,
            dest_admin: false,
        };

        ensure!(
            self.is_connectable(con),
            "the requested connection is not possible"
        );
        ensure!(
            !self.is_connected(con),
            "the requested connection is already made"
        );

        if let Err(e) = self.modules.lock().unwrap()[src_module as usize]
            .1
            .connect(con)
        {
            error!("no connection made. encountered error: {e}");
            bail!(e);
        }

        self.connections.lock().unwrap().push(con);
        self.routing_table.inc_connect_counter(con);

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
            src_admin: false,
            dest_admin: false,
        };

        ensure!(
            self.is_connected(con),
            "the requested connection is possible made, not disconnecting"
        );

        // the counter must be decremented first to avoid the synth seezing up
        self.routing_table.dec_connect_counter(con);
        if let Err(e) = self.modules.lock().unwrap()[src_module as usize]
            .1
            .disconnect(con)
        {
            self.routing_table.inc_connect_counter(con);
            bail!(e);
        }
        self.connections.lock().unwrap().retain(|c| c != &con);

        Ok(())
    }

    /// disconnects all connections
    pub fn disconnect_all(&self) {
        self.connections.lock().unwrap().iter().for_each(|con| {
            self.modules.lock().unwrap().iter().for_each(|module| {
                let _ = module.1.disconnect(con.clone());
            })
        });
        self.connections.lock().unwrap().clear();
        self.routing_table.0.iter().for_each(|mod_ins| {
            mod_ins.iter().for_each(|mod_in| {
                let mut ac = mod_in.active_connections.lock().unwrap();
                *ac = 0;
            })
        });
        self.routing_table.1.iter().for_each(|mod_ins| {
            mod_ins.iter().for_each(|mod_in| {
                let mut ac = mod_in.active_connections.lock().unwrap();
                *ac = 0;
            })
        });
    }

    /// returns `true` if the connection can be made.
    fn is_connectable(&self, connection: Connection) -> bool {
        let mods = self.modules.lock().unwrap();
        // does src_mod exist
        let src_mod = mods.get(connection.src_module as usize).is_some();
        // does src_mod have output
        let src_out = mods.get(connection.src_output as usize).is_some();
        // does dest_mod exist
        let dest_mod = mods.get(connection.dest_module as usize).is_some();
        // does dest_mod have input
        let dest_in = mods.get(connection.dest_input as usize).is_some();

        src_mod && src_out && dest_mod && dest_in
    }

    /// returns `true` if the connection has already been made.
    fn is_connected(&self, connection: Connection) -> bool {
        self.connections.lock().unwrap().contains(&connection)
    }
}
