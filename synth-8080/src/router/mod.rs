use tokio::sync::mpsc;

use crate::{common::Connection, Float};
use std::sync::{Arc, Mutex};

// pub type Inputs = Arc<Mutex<Vec<Float>>>;
// TODO: replace Float with mpsc::Channel
pub type ModuleInputs = Vec<Arc<Mutex<Vec<Option<Float>>>>>;
pub type Inputs = Vec<ModuleInputs>;

#[derive(Default)]
pub struct Modules {
    // pub adbdr: Vec<>,
    // pub adsr: Vec<(Vec<Input>, Vec<Output>)>,
    // pub audio_in: Vec<(Vec<Input>, Vec<Output>)>,
    // pub chorus: Vec<(Vec<Input>, Vec<Output>)>,
    // pub delay: Vec<(Vec<Input>, Vec<Output>)>,
    // pub echo: Vec<(Vec<Input>, Vec<Output>)>,
    // pub gain: Vec<(Vec<Input>, Vec<Output>)>,
    // pub lfo: Vec<(Vec<Input>, Vec<Output>)>,
    // pub mid_pass: Vec<(Vec<Input>, Vec<Output>)>,
    pub output: Option<crate::output::Audio>,
    // pub reverb: Vec<(Vec<Input>, Vec<Output>)>,
    pub vco: Vec<crate::vco::Vco>,
}

pub type Router = Arc<Inputs>;

pub async fn helper_get_idx(router: Router, dest_module: u8, dest_input: u8) -> Option<usize> {
    Some(
        (*router)
            .get(dest_module as usize)?
            .get(dest_input as usize)?
            .lock()
            .unwrap()
            .len(),
    )
}

pub async fn get_idx(router: Router, dest_module: u8, dest_input: u8) -> usize {
    helper_get_idx(router, dest_module, dest_input)
        .await
        .map_or(0, |n| n)
}

pub fn router_set(router: Router, con: Connection, value: Float) -> Option<()> {
    let mut loc = router
        .get(con.dest_module as usize)?
        .get(con.dest_input as usize)?
        .lock()
        .unwrap();

    if con.idx == loc.len() {
        (*loc).push(Some(value));
    } else {
        (*loc)[con.idx as usize] = Some(value);
    }

    Some(())
}
