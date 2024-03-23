use crate::Connection;
use serde::{Deserialize, Serialize};

/// messages emitted by the synth
#[derive(Serialize, Deserialize, PartialEq)]
pub enum SynthRes {
    /// responce to `SynthCmd::GetState`
    SynthState(Vec<Connection>),
    /// a request that the entity connected to the synth identifys its self.
    Identify,
}
