use crate::Float;
pub use lib::notes;
use lib::ModuleType;

// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// pub enum ModuleType {
//     Vco,
//     Output,
//     Lfo,
//     Echo,
//     EnvFilter,
//     Chorus,
//     Delay,
//     OverDrive,
//     Reverb,
//     MCO, // Midi controlled Oscillator
// }

pub struct ModuleInfo {
    pub n_ins: u8,
    pub n_outs: u8,
    pub mod_type: ModuleType,
}

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
// pub struct Connection {
//     pub src_module: u8,
//     pub src_output: u8,
//     pub dest_module: u8,
//     pub dest_input: u8,
// }

pub trait Module {
    /// handles receiving a sample on a designated input
    fn recv_samples(&mut self, input_n: u8, samples: &[Float]);

    /// produces a sample from all outputs
    fn get_samples(&mut self) -> Vec<(u8, Float)>;
}

pub fn bend_range() -> Float {
    (2.0 as Float).powf(2.0 / 12.0)
}
