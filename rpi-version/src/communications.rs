use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, io::Write, sync::Arc};
use strum::{EnumIter, IntoEnumIterator};
use synth_8080::Float;

pub type Message = Vec<u8>;

pub trait ToMessage {
    fn to_message(&self) -> Result<Message>;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, EnumIter)]
pub enum Param {
    Vol,
    EnvAtk,
    EnvDelay,
    EnvSus,
    Cutoff,
    Res,
    ChorusSpeed,
    ChorusVol,
    LfoSpeed,
    LfoVol,
}

impl Into<u8> for Param {
    fn into(self) -> u8 {
        match self {
            Self::Vol => 1,
            Self::EnvAtk => 2,
            Self::EnvDelay => 3,
            Self::EnvSus => 4,
            Self::Cutoff => 5,
            Self::Res => 6,
            Self::ChorusSpeed => 7,
            Self::ChorusVol => 8,
            Self::LfoSpeed => 9,
            Self::LfoVol => 10,
        }
    }
}

impl From<u8> for Param {
    fn from(value: u8) -> Self {
        let selfs = Self::iter();
        let value = (value - 1) % selfs.len() as u8;

        for param in selfs {
            let param_val: u8 = param.into();

            if value == param_val {
                return param;
            }
        }

        unreachable!("if this is seen, please ensure that all params are handled in the param.");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum CtrlInputMesg {
    SetParam(Param, Float),
    Mute(bool),
    // ConLfoTo(Param),
    GetParam(Param),
    // GetLfoCon,
}

impl CtrlInputMesg {
    fn build_type(raw_msg: Arc<[u8]>) -> Result<(Arc<str>, usize)> {
        let mut mesg_type = String::with_capacity(20);

        for (i, b) in raw_msg.iter().enumerate() {
            if *b == 0 as u8 || *b == '\n' as u8 {
                return Ok((mesg_type.into(), i + 1));
            }

            mesg_type = format!("{mesg_type}{}", *b as char);
        }

        bail!("malformed message");
    }

    fn parse_param(raw_msg: Arc<[u8]>, i: usize) -> (Param, usize) {
        (raw_msg[i].into(), i + 1)
    }
}

impl TryFrom<Vec<u8>> for CtrlInputMesg {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        let value: Arc<[u8]> = value.into();

        match Self::build_type(value.clone()) {
            Ok((mesg_type, i)) => {
                if mesg_type.starts_with("SetParam") {
                    let (param, i) = Self::parse_param(value.clone(), i);

                    Ok(Self::SetParam(
                        param,
                        f32::from_le_bytes([value[i], value[1 + 1], value[1 + 2], value[1 + 3]])
                            as Float,
                    ))
                } else if mesg_type.starts_with("Mute") {
                    Ok(Self::Mute(value[i] > 0))
                // } else if mesg_type.starts_with("ConLfoTo") {
                //     let (param, _i) = Self::parse_param(value, i);
                //
                //     Ok(Self::ConLfoTo(param))
                } else if mesg_type.starts_with("GetParam") {
                    let (param, _i) = Self::parse_param(value, i);

                    Ok(Self::GetParam(param))
                // } else if mesg_type.starts_with("GetLfoCon") {
                //     Ok(Self::GetLfoCon)
                } else {
                    bail!("unknown Input message recieved")
                }
            }
            Err(e) => bail!("parsing raw message failed with error: {e}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum CtrlResponseMesg {
    /// what param was set
    SetSuccess(Param),
    /// if the synth is muted
    Muted(bool),
    // /// Lfo Connected to param success
    // LfoConSuccess(Param),
    /// a parameters value
    ParamVal(Param, f32),
    // /// tells the controller what the lfo is wired to
    // LfoCon(Param),
}

impl Display for CtrlResponseMesg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetSuccess(_) => write!(f, "SetSuccess"),
            Self::Muted(_) => write!(f, "Muted"),
            // Self::LfoConSuccess(_) => write!(f, "LfoConSucces"),
            Self::ParamVal(_, _) => write!(f, "ParamVal"),
            // Self::LfoCon(_) => write!(f, "LfoCon"),
        }
    }
}

impl ToMessage for CtrlResponseMesg {
    fn to_message(&self) -> Result<Message> {
        let mut f: Vec<u8> = Vec::with_capacity(20);

        match self {
            Self::SetSuccess(param) => {
                write!(f, "SetSuccess")?;
                f.push(0);
                f.push((*param).into());
                f.push('\n' as u8);
            }
            Self::Muted(param) => {
                write!(f, "Muted")?;
                f.push(0);
                f.push((*param).into());
                f.push('\n' as u8);
            }
            // Self::LfoConSuccess(param) => {
            //     write!(f, "LfoConSucces")?;
            //     f.push(0);
            //     f.push((*param).into());
            //     f.push('\n' as u8);
            // }
            Self::ParamVal(param, val) => {
                write!(f, "ParamVal")?;
                f.push(0);
                f.push((*param).into());
                f.push(0);
                f.append(&mut val.to_le_bytes().to_vec());
                f.push('\n' as u8);
            } // Self::LfoCon(param) => {
              //     write!(f, "LfoCon")?;
              //     f.push(0);
              //     f.push((*param).into());
              //     f.push('\n' as u8);
              // }
        }

        Ok(f)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn responce_message_ser() {
        // TODO: Write
        todo!();
    }

    #[test]
    fn input_message_deser() {
        // TODO: Write
        todo!();
    }
}
