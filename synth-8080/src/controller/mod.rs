use crate::{
    common::{Connection, Module, ModuleInfo, ModuleType},
    router::{Router, RoutingTable},
};
use anyhow::ensure;
use futures::future::join_all;
use std::sync::{Arc, Mutex};
use tokio::{spawn, task::JoinHandle};
use tracing::info;

pub struct Controller {
    /// the liist of connections
    pub connections: Arc<Mutex<Vec<Connection>>>,
    /// the list of registered modules
    pub modules: Arc<Mutex<Vec<(ModuleInfo, Box<dyn Module>)>>>,
    /// a table representing all inputs of all modules
    // is already an Arc
    pub routing_table: Router,
    pub handles: Vec<JoinHandle<()>>,
    // TODO: find a serial library to talk to the micro controller
    // /// UART connectino to the micro-controller
    // pub serial: Arc<Mutex<>>,
}

impl Controller {
    pub async fn new(to_build: &[ModuleType]) -> anyhow::Result<Self> {
        let connections = Vec::new();
        let info = to_build.iter().map(|mod_type| mod_type.get_info());

        let routing_table: Router = Arc::new(info.clone().map(|mod_type| mod_type.io).collect());
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
        .map(|(m, i)| (i, m))
        .collect();
        info!("made module list");
        // make modules

        // start modules
        let handles = modules
            .iter()
            .map(|(_info, module)| module.start())
            .collect::<anyhow::Result<Vec<JoinHandle<()>>>>()?;
        info!("started the modules");

        Ok(Self {
            connections: Arc::new(Mutex::new(connections)),
            modules: Arc::new(Mutex::new(modules)),
            routing_table,
            handles,
        })
    }

    pub fn start(&self) -> JoinHandle<()> {
        // TODO: trun LED red
        spawn(async move {
            // TODO: handle serial evvents from micro controller
        })
    }

    pub fn register(&self, module: Box<dyn Module>, info: ModuleInfo) {
        self.modules.lock().unwrap().push((info, module));
    }

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

        self.modules.lock().unwrap()[src_module as usize]
            .1
            .connect(con)?;
        self.routing_table.inc_connect_counter(con);
        self.connections.lock().unwrap().push(con);

        Ok(())
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
