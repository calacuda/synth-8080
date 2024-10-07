use esp_idf_svc::sys::{esp, esp_vfs_dev_uart_use_driver, uart_driver_install};
use esp_idf_svc::hal::prelude::*;
use log::*;
use std::ptr::null_mut;
use std::io::{Read, Write};
use lib::communication::command::SynthCmd;

pub mod sm;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("taking peripherals");
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    info!("peripherals taken");

    uart_init();

    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    // idenfy 
    loop {
        // TODO: wait for itentify request from synth
        let mut buffer = Vec::new();

        stdin.read(&mut buffer)?;

        if lib::communication::responce::SynthRes::Identify == serde_cbor::from_slice(&buffer)? {
            stdout.write_all(&serde_cbor::to_vec(&SynthCmd::Identify(lib::communication::command::SynthId::ControlsBoard))?)?;
            break;
        } else {
            // error!("not identify");
        }
    }

    loop {
        // TODO: update statemachine
        // TODO: print updates to stdout
    }

    // Ok(())
}

fn uart_init() {
    info!("initing UART driver for reading");
    unsafe {
        let uart_num = 0;
        esp!(uart_driver_install(uart_num, 512, 512, 10, null_mut(), 0)).unwrap();
        esp_vfs_dev_uart_use_driver(uart_num);
    }
    info!("initing UART driver for reading");
}
