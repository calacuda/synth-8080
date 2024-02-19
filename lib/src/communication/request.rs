use super::command::SynthCmd;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum SynthReq {
    /// gets the entire, absolute state of the synth
    GetState,
    /// sends a valid synth command to the synth
    Command(SynthCmd),
}
