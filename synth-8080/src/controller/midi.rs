use super::Note;
use anyhow::bail;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiInputPort};
use midly::{live::LiveEvent, MidiMessage};
use std::{
    future::Future,
    io,
    process::exit,
    str::FromStr,
    sync::{Arc, Mutex},
    task::Poll,
};
use tracing::*;

pub struct MIDIControls {
    controller: Arc<super::Controller>,
    /// MIDI connection to the micro-controller
    midi_cons: Vec<MidiInputConnection<()>>,
    // midi_in: MidiInput,
}

impl MIDIControls {
    pub fn new(controller: Arc<super::Controller>) -> anyhow::Result<Self> {
        info!("making midi controller");

        Ok(Self {
            controller,
            midi_cons: Vec::new(),
            // midi_in,
        })
    }

    fn mk_midi_input(&mut self) -> anyhow::Result<MidiInput> {
        let mut midi_in = MidiInput::new("midir reading input")?;
        midi_in.ignore(Ignore::None);
        info!("midi_in made");

        Ok(midi_in)
    }

    pub fn list_ports(&mut self) -> anyhow::Result<Vec<(MidiInputPort, String)>> {
        let mut midi_in = self.mk_midi_input()?;

        // Get an input port (read from console if multiple are available)
        Ok(midi_in
            .ports()
            .into_iter()
            .map(|port| Ok((port.clone(), midi_in.port_name(&port)?)))
            .collect::<anyhow::Result<Vec<(MidiInputPort, String)>>>()?)
    }

    pub fn connect_to_port(&mut self, in_port: MidiInputPort) -> anyhow::Result<()> {
        let mut midi_in = self.mk_midi_input()?;

        let in_port_name = midi_in.port_name(&in_port)?;
        info!("Opening connection to midi port: {in_port_name}");
        let ctrlr = self.controller.clone();

        // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
        let midi_con = midi_in
            .connect(
                &in_port,
                "midir-read-input",
                // &in_port_name,
                move |stamp, message, _| {
                    // info!("{}: {:?} (len = {})", stamp, message, message.len());
                    let event = LiveEvent::parse(message).unwrap();

                    match event {
                        LiveEvent::Midi { channel, message } => match message {
                            MidiMessage::NoteOn { key, vel } => {
                                // info!("hit note {} on channel {}", key, channel);
                                let note = Note::from(u8::from(key));
                                // if let Ok(note) = Note::from(&key) {
                                info!("playing {note}");
                                ctrlr.modules.lock().unwrap().mco[0].play_note(note);
                                // } else {
                                //     error!("{}", key.to_string());
                                // }
                            }
                            MidiMessage::NoteOff { key, vel } => {
                                // info!("released note {} on channel {}", key, channel);
                                let note = Note::from(u8::from(key));
                                // if let Ok(note) = Note::from(&key) {
                                info!("stopping {note}");
                                ctrlr.modules.lock().unwrap().mco[0].stop_note(note);
                                // } else {
                                //     error!("{}", key.to_string());
                                // }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                    // trace!("concluded midi function.")
                },
                (),
            )
            .map_or_else(|e| bail!("midi connection error: {e}"), |con| Ok(con))?;
        // info!("midi connected");

        self.midi_cons.push(midi_con);
        info!("Connected to midi port: {in_port_name}");

        Ok(())
    }

    pub fn connect_default(&mut self) -> anyhow::Result<()> {
        // info!("making midi controller");
        // let mut midi_in = MidiInput::new("midir reading input")?;
        // midi_in.ignore(Ignore::None);
        // info!("midi_in made");
        //
        // // Get an input port (read from console if multiple are available)
        // let in_ports = midi_in.ports();
        // info!("in_ports => {}", in_ports.len());
        let ports = self.list_ports()?;

        if ports.len() > 1 {
            let in_port = ports[1].0.clone();
            self.connect_to_port(in_port)?;
        } else {
            let in_port = ports[0].0.clone();
            self.connect_to_port(in_port)?;
        }
        // .into_iter()
        // .map(|port| self.connect_to_port(port.0))
        // .collect::<anyhow::Result<()>>()?;

        // let in_port = match in_ports.len() {
        //     0 => bail!("no input port found"),
        //     1 => {
        //         info!(
        //             "Choosing the only available input port: {}",
        //             midi_in.port_name(&in_ports[0]).unwrap()
        //         );
        //         &in_ports[0]
        //     }
        //     2 => {
        //         info!(
        //             "Choosing the second available input port: {}",
        //             midi_in.port_name(&in_ports[1]).unwrap()
        //         );
        //         &in_ports[1]
        //     }
        //     _ => bail!("too many midi devices. exiting."),
        // };

        // info!("Opening connection");
        // let in_port_name = midi_in.port_name(in_port)?;
        // let ctrlr = self.controller.clone();
        //
        // // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
        // let midi_con = midi_in
        //     .connect(
        //         in_port,
        //         "midir-read-input",
        //         // &in_port_name,
        //         move |stamp, message, _| {
        //             info!("{}: {:?} (len = {})", stamp, message, message.len());
        //             let event = LiveEvent::parse(message).unwrap();
        //
        //             match event {
        //                 LiveEvent::Midi { channel, message } => match message {
        //                     MidiMessage::NoteOn { key, vel } => {
        //                         info!("hit note {} on channel {}", key, channel);
        //                         let note = Note::from(u8::from(key));
        //                         // if let Ok(note) = Note::from(&key) {
        //                         info!("playing {note}");
        //                         ctrlr.modules.lock().unwrap().mco[0].play_note(note);
        //                         // } else {
        //                         //     error!("{}", key.to_string());
        //                         // }
        //                     }
        //                     MidiMessage::NoteOff { key, vel } => {
        //                         info!("released note {} on channel {}", key, channel);
        //                         let note = Note::from(u8::from(key));
        //                         // if let Ok(note) = Note::from(&key) {
        //                         info!("stopping {note}");
        //                         ctrlr.modules.lock().unwrap().mco[0].stop_note(note);
        //                         // } else {
        //                         //     error!("{}", key.to_string());
        //                         // }
        //                     }
        //                     _ => {}
        //                 },
        //                 _ => {}
        //             }
        //             // trace!("concluded midi function.")
        //         },
        //         (),
        //     )
        //     .map_or_else(|e| bail!("midi connection error: {e}"), |con| Ok(con))?;
        // info!("midi connected");
        //
        // self.midi_cons.push(midi_con);

        Ok(())
    }

    // fn handle_midi_in(&self, message: &[u8]) {
    //         }
}

// impl Future for HardwareControls {
//     type Output = ();
//
//     fn poll(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//         // let mut serial_buf: Vec<u8> = vec![0; 1000];
//         //
//         // match self.serial.lock().unwrap().read(serial_buf.as_mut_slice()) {
//         //     Ok(t) => {
//         //         let raw_input = String::from_utf8_lossy(&serial_buf[..t]);
//         //         let cmd = raw_input.trim();
//         //
//         //         info!("received command: {cmd:?}");
//         //
//         //         if cmd == "play" {
//         //             info!("setting Notes");
//         //             self.controller.play(Note::A4);
//         //         } else if cmd == "stop" {
//         //             self.controller.stop(Note::A4);
//         //         } else if cmd == "power-off" {
//         //             cx.waker().wake_by_ref();
//         //             return Poll::Ready(());
//         //         }
//         //     }
//         //     Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
//         //     Err(e) => error!("{:?}", e),
//         // }
//
//         cx.waker().wake_by_ref();
//         Poll::Pending
//     }
// }
