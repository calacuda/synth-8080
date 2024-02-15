use crate::{
    common::{notes::Note, Connection, ModuleType},
    output,
    router::Modules,
    JoinHandle,
};
use anyhow::ensure;
use crossbeam_channel::{unbounded, Receiver};
use std::sync::Mutex;
use tracing::*;

pub mod harware;

pub struct Controller {
    /// the liist of connections
    pub connections: Mutex<Vec<Connection>>,
    pub modules: Mutex<Modules>,
    pub output: Mutex<output::Output>,
    pub sync: Receiver<()>,
    // TODO: find lib to talk to MIDI device
    // /// Connection to MIDI device
    // pub midi: Arc<Mutex<>>,
    pub playing: Mutex<Vec<(usize, Note)>>,
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
                playing: Mutex::new(Vec::new()),
            },
            jh,
        ))
    }

    pub fn play(&self, note: Note) {
        let mut playing = self.playing.lock().unwrap();

        if playing
            .iter()
            .filter(|(_, n)| *n == note)
            .peekable()
            .peek()
            .is_none()
        {
            let mut mods = self.modules.lock().unwrap();

            if let Some(i) = mods.vco.iter_mut().enumerate().find_map(|(i, f)| {
                if f.osc.frequency == 0.0 {
                    Some(i)
                } else {
                    None
                }
            }) {
                mods.vco[i].set_note(note);
                playing.push((i, note));
                mods.filter[i].envelope.open_filter(vec![1.0]);
            } else {
                error!("already playing notes");
            }
        } else {
            error!("note {note} is already being played");
        }
    }

    pub fn stop(&self, note: Note) {
        if self
            .playing
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, n)| *n == note)
            .peekable()
            .peek()
            .is_some()
        {
            let mut mods = self.modules.lock().unwrap();

            if let Some(i) = mods.vco.iter_mut().enumerate().find_map(|(i, f)| {
                if f.osc.frequency == note.into() {
                    Some(i)
                } else {
                    None
                }
            }) {
                mods.vco[i].osc.set_frequency(0.0);
                mods.filter[i].envelope.open_filter(vec![0.0]);
            } else {
                error!("already playing notes");
            }
        } else {
            error!("note {note} is already being played");
        }

        self.playing.lock().unwrap().retain(|(_, n)| *n != note);
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
