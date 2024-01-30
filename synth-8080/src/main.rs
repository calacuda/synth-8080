use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use prettytable::{row, Table};
use std::mem;
use std::str::FromStr;
use std::{fmt::Display, net::IpAddr};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub mod adbdr;
pub mod adsr;
pub mod chorus;
pub mod common;
pub mod delay;
pub mod echo;
pub mod lfo;
pub mod mid_pass;
pub mod osc;
pub mod reverb;
pub mod vco;

// pub type Float = f32;
pub type Float = f64;
pub const SAMPLE_RATE: u32 = 48_000;
pub const FLOAT_LEN: usize = mem::size_of::<Float>();

/// used by clap to handle cmd line args
#[derive(Clone, Debug, EnumIter)]
enum NodeType {
    // /// runs the rest of the modules and reads data from the controls via uart
    // Operator,
    /// ADBDR envelope filter
    ADBDR,
    /// ADSR envelope filter
    ADSR,
    /// Chourus effect
    Chorus,
    /// Delay effect
    Delay,
    /// Echo effect
    Echo,
    /// LFO (Low Frequency Oscilator) node
    LFO,
    /// Mid-Pass filter
    MidPass,
    /// Reverb effect
    Reverb,
    /// VCO (Voltage controlled oscilator)
    VCO,
}

impl FromStr for NodeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            // "operator" => Ok(NodeType::Operator),
            "adbdr" => Ok(NodeType::ADBDR),
            "adsr" => Ok(NodeType::ADSR),
            "chorus" => Ok(NodeType::Chorus),
            "delay" => Ok(NodeType::Delay),
            "echo" => Ok(NodeType::Echo),
            "lfo" => Ok(NodeType::LFO),
            "mid-pass" | "band-pass" => Ok(NodeType::MidPass),
            "reverb" => Ok(NodeType::Reverb),
            "vco" => Ok(NodeType::VCO),
            _ => Err(format!("\"{s}\" is not a valid module name")),
        }
    }
}

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ADBDR => write!(f, "adbdr"),
            Self::ADSR => write!(f, "adsr"),
            Self::Chorus => write!(f, "chorus"),
            Self::Delay => write!(f, "delay"),
            Self::Echo => write!(f, "echo"),
            Self::LFO => write!(f, "lfo"),
            Self::MidPass => write!(f, "mid-pass || band-pass"),
            Self::Reverb => write!(f, "reverb"),
            Self::VCO => write!(f, "vco"),
        }
    }
}

impl NodeType {
    pub fn describe(&self) -> (String, String) {
        let name = format!("{self}");

        let desc = match *self {
            Self::ADBDR => "Attack-Decay-Break-Decay2-Release, envelope filter",
            Self::ADSR => "Attack-Decay-Sustain-Release, envelope filter",
            Self::Chorus => "Chorus effect",
            Self::Delay => "Delay effect",
            Self::Echo => "Echo effect",
            Self::LFO => "Low Frequency Oscilator, used to controle parameters of other modules",
            Self::MidPass => "A Mid-Pass filter",
            Self::Reverb => "Reverb effect",
            Self::VCO => "Voltage Controlled Oscilator, produces an audio signal",
        };

        (name, desc.to_string())
    }
}

#[derive(Args, Clone, Debug)]
// #[command(version, about, long_about = None)]
struct NodeArgs {
    /// module to start
    #[arg()]
    module: NodeType,

    /// the ip address of the Operator node to register with
    #[arg()]
    address: IpAddr,

    /// the port the operator program is running on
    #[arg(default_value_t = 8080)]
    port: u16,
}

#[derive(Subcommand, Clone, Debug)]
enum SubCmd {
    /// lists all valid audio nodes
    #[clap(visible_alias = "ls")]
    List,
    /// makes a new audio module
    #[clap(visible_alias = "module")]
    New(NodeArgs),
    /// starts the synthesizer, should only be called once
    #[clap(visible_alias = "boot")]
    Start,
}

// impl FromStr for SubCmd {
//     type Err = String;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "new" | "mod" | "module" => Ok(Self::New),
//             "list" | "ls" | "nodes" => Ok(Self::ListNodes),
//             "start" | "boot" => Ok(Self::Start),
//             _ => Err(format!("\"{s}\" is not a valid sub-command")),
//         }
//     }
// }

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: SubCmd,
}

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: get cmd line args
    let args = Cli::parse();

    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_thread_ids(true)
        .with_target(true)
        .without_time()
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber)?;

    match args.cmd {
        SubCmd::New(args) => new_node(args).await?,
        SubCmd::Start => start().await?,
        SubCmd::List => list_modules().await?,
    }

    Ok(())
}

/// spins up a new audio module
async fn new_node(args: NodeArgs) -> Result<()> {
    // let args = NodeArgs::parse();

    match args.module {
        NodeType::VCO => vco::start().await?,
        _ => {}
    }
    // start server

    Ok(())
}

/// used to start a synthesiser, should be called only once
async fn start() -> Result<()> {
    Ok(())
}

/// lists audio modules so the user knows what is supported
async fn list_modules() -> Result<()> {
    let mut table = Table::new();
    table.set_titles(row!["Flag", "Description"]);
    table.get_format().borders(' ');

    NodeType::iter().for_each(|module| {
        let (flag, desc) = module.describe();
        table.add_row(row![flag, desc]);
    });

    table.printstd();
    Ok(())
}
