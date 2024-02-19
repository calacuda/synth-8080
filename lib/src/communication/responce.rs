use crate::Connection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum SynthRes {
    SynthState(Vec<Connection>),
}
