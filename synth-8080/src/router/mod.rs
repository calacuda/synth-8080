use crate::{Float, NodeType};
use std::sync::{Arc, Mutex};

// pub type Inputs = Arc<Mutex<Vec<Float>>>;
pub type Outputs = Arc<Mutex<[Option<Float>; 16]>>;
pub type Inputs = Arc<Mutex<[Option<Float>; 16]>>;

// pub struct Rouuter {
//     pub adbdr: Vec<(Vec<Input>, Vec<Output>)>,
//     pub adsr: Vec<(Vec<Input>, Vec<Output>)>,
//     pub audio_in: Vec<(Vec<Input>, Vec<Output>)>,
//     pub chorus: Vec<(Vec<Input>, Vec<Output>)>,
//     pub delay: Vec<(Vec<Input>, Vec<Output>)>,
//     pub echo: Vec<(Vec<Input>, Vec<Output>)>,
//     pub gain: Vec<(Vec<Input>, Vec<Output>)>,
//     pub lfo: Vec<(Vec<Input>, Vec<Output>)>,
//     pub mid_pass: Vec<(Vec<Input>, Vec<Output>)>,
//     pub output: Vec<(Vec<Input>, Vec<Output>)>,
//     pub reverb: Vec<(Vec<Input>, Vec<Output>)>,
//     pub vco: Vec<(Vec<Input>, Vec<Output>)>,
// }

pub type Router = [(NodeType, Inputs, Outputs); 255];
