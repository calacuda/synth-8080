use crate::{
    common::{notes::Note, Connection, Module, ModuleType},
    output,
    router::Modules,
    spawn, JoinHandle,
};
use anyhow::{ensure, Result};
use crossbeam_channel::{unbounded, Receiver};
use serialport::SerialPort;
use std::{
    io,
    sync::{Arc, Mutex},
};
use tracing::*;

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
    connections: Arc<Mutex<Vec<Connection>>>,
    modules: Arc<Mutex<Modules>>,
    output: output::Output,
    sync: Receiver<()>,
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
    /// UART connection to the micro-controller
    pub serial: Arc<Mutex<Box<dyn SerialPort>>>,
    // pub output: Arc<Output>,
    // TODO: find lib to talk to MIDI device
    // /// Connection to MIDI device
    // pub midi: Arc<Mutex<>>,
}

impl Controller {
    pub async fn new(to_build: &[ModuleType]) -> anyhow::Result<(Self, JoinHandle)> {
        // let connections = Vec::new();
        // let info = to_build.iter().map(|mod_type| mod_type.get_info());
        //
        // let normal_name_space: Vec<(Arc<[ModuleIn]>, (Sender<()>, Receiver<()>))> = info
        //     .clone()
        //     .map(|mod_type| (mod_type.0.io.into(), unbounded::<()>()))
        //     .collect();
        // let admin_name_space: Vec<(Arc<[ModuleIn]>, (Sender<()>, Receiver<()>))> = info
        //     .clone()
        //     .map(|mod_type| (mod_type.0.io.into(), unbounded::<()>()))
        //     .collect();
        //
        // let (global_sync_tx, global_sync_rx) = unbounded();
        //
        // let routing_table: Router = Arc::new(Inputs {
        //     in_s: normal_name_space.into(),
        //     admin_in_s: admin_name_space.into(),
        //     sync: global_sync_rx,
        // });
        // info!("made routing table");
        // // TODO: make output
        // let mut modules: Vec<(ModuleInfo, Box<dyn Module>)> = vec![(
        //     ModuleInfo {
        //         n_ins: 1,
        //         n_outs: 0,
        //         io: mk_module_ins(1),
        //         mod_type: ModuleType::Output,
        //     },
        //     Box::new(Output::new(routing_table.clone(), global_sync_tx)),
        // )];
        //
        // let mut mods: Vec<(ModuleInfo, Box<dyn Module>)> = join_all(
        //     to_build[0..]
        //         .iter()
        //         // .zip(info)
        //         .enumerate()
        //         .map(|(i, mod_type)| mod_type.builder(routing_table.clone(), i)),
        // )
        // .await
        // .into_iter()
        // .zip(info)
        // .filter_map(|(m, i)| {
        //     if let Some(m) = m {
        //         Some((i.0, m))
        //     } else {
        //         None
        //     }
        // })
        // .collect();
        // modules.append(&mut mods);
        //
        // info!("made module list");
        // // make modules
        //
        // // start modules
        // let handles = Arc::new(Mutex::new(
        //     modules
        //         .iter()
        //         .map(|(_info, module)| module.start())
        //         .collect::<anyhow::Result<Vec<JoinHandle>>>()?,
        // ));
        // // (*handles.lock().unwrap().deref_mut()).push(output.start()?);
        // info!("started the modules");
        //
        let serial = Arc::new(Mutex::new(serialport::new("/dev/ttyACM0", 115200).open()?));
        // let oscilators = Arc::new(Mutex::new(OscilatorId::new_s(to_build)?));
        let (tx, sync) = unbounded();
        let (output, jh) = output::Output::new(tx);
        let modules = Arc::new(Mutex::new(Modules::from(to_build)));

        Ok((
            Self {
                connections: Arc::new(Mutex::new(Vec::new())),
                modules,
                serial,
                sync,
                output,
                // jh,
                // src_s: HashSet::new(),
                // dest_s: HashSet::new(),
            },
            jh,
        ))
    }

    /// starts an event loop to listen for events over both serial and MIDI.
    pub fn start_harware(&mut self) -> JoinHandle {
        //     info!("starting controller event loop");
        //     // TODO: trun LED red

        let mut serial_buf: Vec<u8> = vec![0; 1000];
        let port = self.serial.clone();
        let modules = self.modules.clone();

        spawn(async move {
            loop {
                // trace!("inside controller serial read loop");
                match port.lock().unwrap().read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        let raw_input = String::from_utf8_lossy(&serial_buf[..t]);
                        let cmd = raw_input.trim();

                        info!("recieved command: {cmd:?}");

                        if cmd == "play" {
                            info!("setting Notes");
                            modules.lock().unwrap().vco[0].set_note(Note::A4);
                            modules.lock().unwrap().filter[0]
                                .envelope
                                .open_filter(vec![1.0]);
                            // let mut os = note.lock().unwrap();
                            // *os = Note::A4.into();
                            //
                            // let mut es = filter_open.lock().unwrap();
                            // *es = 1.0;
                        } else if cmd == "stop" {
                            modules.lock().unwrap().filter[0]
                                .envelope
                                .open_filter(vec![0.0]);
                            // modules.lock().unwrap().vco[0].set_note;
                            // let mut os = note.lock().unwrap();
                            // *os = 0.0;
                            //
                            // let mut es = filter_open.lock().unwrap();
                            // *es = 0.0;
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        })
    }

    pub async fn step(&mut self) {
        loop {
            // warn!("foobar");
            if let Err(e) = self.sync.recv() {
                error!("error recieving sync message: {e}");
            };

            let mut src_samples = [[0.0; 16]; u8::MAX as usize];

            for src in 0..u8::MAX as usize {
                if let Some(mods) = self.modules.lock().unwrap().get_output(src).await {
                    // println!("foobar");
                    // warn!("mod_type {} => {:?}", src, self.modules.indeces.get(src));
                    mods.into_iter()
                        .for_each(|(output, sample)| src_samples[src][output as usize] += sample);
                } else {
                    break;
                }
            }

            let mut dest_samples = [[0.0; 16]; u8::MAX as usize];
            let mut destinations: Vec<(u8, u8)> = Vec::with_capacity(256);

            for con in self.connections.lock().unwrap().iter() {
                dest_samples[con.dest_module as usize][con.dest_input as usize] +=
                    src_samples[con.src_module as usize][con.src_output as usize];

                let dest = (con.dest_module, con.dest_input);

                if !destinations.contains(&dest) {
                    destinations.push(dest);
                }
            }

            for (dest_mod, dest_in) in destinations {
                let sample = dest_samples[dest_mod as usize][dest_in as usize];

                if dest_mod == 0 {
                    self.output.recv_samples(0, &vec![sample]).await;
                } else {
                    self.modules
                        .lock()
                        .unwrap()
                        .send_sample_to(dest_mod as usize, dest_in as usize, &vec![sample])
                        .await;
                }
            }

            // for con in self.connections.lock().unwrap().iter() {
            //     let sample = dest_samples[con.dest_module as usize][con.dest_input as usize];
            //
            //     if con.dest_module == 0 {
            //         self.output.recv_samples(0, &vec![sample]).await;
            //     } else {
            //         self.modules
            //             .lock()
            //             .unwrap()
            //             .send_sample_to(
            //                 con.dest_module as usize,
            //                 con.dest_input as usize,
            //                 &vec![sample],
            //             )
            //             .await;
            //     }
            // }
        }
    }

    /// connects src module to dest module
    pub fn connect(
        &mut self,
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
        &mut self,
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
    pub fn disconnect_all(&mut self) {
        self.connections.lock().unwrap().clear();
    }

    /// returns `true` if the connection can be made.
    fn is_connectable(&mut self, _connection: Connection) -> bool {
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
