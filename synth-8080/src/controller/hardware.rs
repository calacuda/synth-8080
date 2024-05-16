use super::Note;
use serialport::SerialPort;
use std::{
    future::Future,
    io,
    sync::{Arc, Mutex},
    task::Poll,
};
use tracing::*;

pub struct HardwareControls {
    controller: Arc<super::Controller>,
    /// UART connection to the micro-controller
    serial: Mutex<Box<dyn SerialPort>>,
}

impl HardwareControls {
    pub fn new(controller: Arc<super::Controller>) -> anyhow::Result<Self> {
        let serial = Mutex::new(serialport::new("/dev/ttyACM0", 115200).open()?);
        serial
            .lock()
            .unwrap()
            .clear(serialport::ClearBuffer::Output)?;
        let mut serial_buf: Vec<u8> = vec![0; 1000];
        if let Err(e) = serial.lock().unwrap().read(&mut serial_buf) {
            if e.kind() != io::ErrorKind::TimedOut {
                error!("error clearing buff: {e}")
            };
        }
        info!("serial controller ready...");

        Ok(HardwareControls { controller, serial })
    }
}

impl Future for HardwareControls {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // let me = self;
        let mut serial_buf: Vec<u8> = vec![0; 1000];

        match self.serial.lock().unwrap().read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                let raw_input = String::from_utf8_lossy(&serial_buf[..t]);
                let cmd = raw_input.trim();

                info!("received command: {cmd:?}");

                if cmd == "play" {
                    info!("setting Notes");
                    self.controller.play(Note::A4);
                } else if cmd == "stop" {
                    self.controller.stop(Note::A4);
                } else if cmd == "power-off" {
                    cx.waker().wake_by_ref();
                    return Poll::Ready(());
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => error!("{:?}", e),
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
