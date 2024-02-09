use crate::{
    common::{event_loop, notes::Note, Connection, Module, ModuleInfo, ModuleType},
    envelope,
    router::{ModuleIn, Router, RoutingTable},
    vco, Float,
};
use anyhow::{bail, ensure, Result};
use futures::future::join_all;
use serialport::SerialPort;
use std::{
    io,
    sync::{Arc, Mutex},
};
use tokio::{spawn, task::JoinHandle};
use tracing::{error, info, warn};

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
    pub connections: Arc<Mutex<Vec<Connection>>>,
    /// the list of registered modules
    pub modules: Arc<Mutex<Vec<(ModuleInfo, Box<dyn Module>)>>>,
    /// a table representing all inputs of all modules
    pub routing_table: Router,
    /// a list of join handles for the event loops of all modules
    pub handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    /// list of the locations of the oscilators and coresponding envelope filters
    pub oscilators: Arc<Mutex<Vec<OscilatorId>>>,
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

        let normal_name_space: Vec<Arc<[ModuleIn]>> =
            info.clone().map(|mod_type| mod_type.0.io.into()).collect();
        let admin_name_space: Vec<Arc<[ModuleIn]>> =
            info.clone().map(|mod_type| mod_type.0.io.into()).collect();
        let routing_table: Router = Arc::new((normal_name_space.into(), admin_name_space.into()));
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
        let oscilators = Arc::new(Mutex::new(OscilatorId::new_s(to_build)?));

        Ok(Self {
            connections: Arc::new(Mutex::new(connections)),
            modules: Arc::new(Mutex::new(modules)),
            routing_table,
            handles,
            oscilators,
            serial,
        })
    }

    /// starts an event loop to listen for events over both serial and MIDI.
    pub fn start(&self) -> JoinHandle<()> {
        info!("starting controller event loop");
        // TODO: trun LED red

        let port = self.serial.clone();
        let osc = self.oscilators.clone();
        let router = self.routing_table.clone();
        let connections = self.connections.clone();
        let handles = self.handles.clone();
        // handle serial events from micro controller
        // sleep(Duration::from_secs(1)).await
        let osc_sample = Arc::new(Mutex::new(0.0));
        let env_sample = Arc::new(Mutex::new(0.0));
        let note = osc_sample.clone();
        let filter_open = env_sample.clone();

        let (osc_id, env_id) = osc.lock().unwrap()[0].get();
        // info!("osc id => {osc_id}, env id => {env_id}");

        let gen_env_sample: Box<dyn FnMut() -> Float + Send> = Box::new(move || {
            let sample = *env_sample.lock().unwrap();
            // info!("envelope filter is open: {}", sample >= 0.75);

            sample
        });
        let jh = Controller::spawn_admin_cmd(
            gen_env_sample,
            env_id as u8,
            envelope::FILTER_OPEN_IN,
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

        let mut serial_buf: Vec<u8> = vec![0; 1000];

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
            let ins: Arc<[ModuleIn]> = (*router)
                .1
                .get(dest_module as usize)
                .expect("this VCO Module was not found in the routing table struct.")
                .clone();
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
