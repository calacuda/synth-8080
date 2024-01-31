use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use prettytable::{row, Table};
use serde::{Deserialize, Serialize};
use std::mem;
use std::str::FromStr;
use std::{
    fmt::Display,
    net::{IpAddr, Ipv4Addr},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tracing::info;

pub mod adbdr;
pub mod adsr;
pub mod audio_in;
pub mod chorus;
pub mod common;
pub mod controller;
pub mod delay;
pub mod echo;
pub mod gain;
pub mod lfo;
pub mod mid_pass;
pub mod osc;
pub mod output;
pub mod reverb;
pub mod vco;

// pub type Float = f32;
pub type Float = f64;
pub const SAMPLE_RATE: u32 = 48_000;
pub const FLOAT_LEN: usize = mem::size_of::<Float>();

/// used by clap to handle cmd line args
#[derive(Clone, Debug, EnumIter, Deserialize, Serialize)]
pub enum NodeType {
    /// ADBDR envelope filter
    ADBDR,
    /// ADSR envelope filter
    ADSR,
    /// Retrieves the audio source data from the audio jack
    AudioIn,
    /// Chourus effect
    Chorus,
    /// Delay effect
    Delay,
    /// Echo effect
    Echo,
    /// Gain attenuator,
    Gain,
    /// LFO (Low Frequency Oscilator) node
    LFO,
    /// Mid-Pass filter
    MidPass,
    /// Handles sending audio output to the speakers
    Output,
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
            Self::AudioIn => write!(f, "audio-in"),
            Self::Chorus => write!(f, "chorus"),
            Self::Delay => write!(f, "delay"),
            Self::Echo => write!(f, "echo"),
            Self::Gain => write!(f, "gain"),
            Self::LFO => write!(f, "lfo"),
            Self::MidPass => write!(f, "mid-pass || band-pass"),
            Self::Output => write!(f, "output || audio-out || out"),
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
            Self::AudioIn => "Audio source data from the built in audio jack",
            Self::Chorus => "Chorus effect",
            Self::Delay => "Delay effect",
            Self::Echo => "Echo effect",
            Self::Gain => "Gain attenuator for multiple inputs",
            Self::LFO => "Low Frequency Oscilator, used to controle parameters of other modules",
            Self::MidPass => "A Mid-Pass filter",
            Self::Output => "Outputs audio to the speakers",
            Self::Reverb => "Reverb effect",
            Self::VCO => "Voltage Controlled Oscilator, produces an audio signal",
        };

        (name, desc.to_string())
    }
}

#[derive(Args, Clone, Debug)]
// #[command(version, about, long_about = None)]
pub struct NodeArgs {
    /// module to start
    #[arg()]
    module: NodeType,

    /// the ip address of the Operator node to register with
    #[arg()]
    op_adr: IpAddr,

    #[arg(default_value_t = 8080)]
    mod_port: u16,

    /// the port the operator program is running on
    #[arg(default_value_t = 8080)]
    op_port: u16,
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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: SubCmd,
}

#[tokio::main]
async fn main() -> Result<()> {
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
    let bind_ip = "127.0.0.1";
    // TODO: set bind_ip to docker bridge ip

    let mod_conf = controller::Module {
        ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1")?),
        port: args.mod_port,
        kind: args.module.clone(),
    };
    let url = format!("http://{}:{}/register/", args.op_adr, args.op_port);
    info!("registering with {url}");
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .header("Content-Type", "text/json")
        .body(serde_json::to_string(&mod_conf)?)
        .send()
        .await?;
    info!("register responce status code: {}", res.status());

    match args.module {
        NodeType::VCO => {
            // vco::register(&args).await?;
            vco::start(bind_ip, args.mod_port).await?;
        }
        _ => {}
    }
    // start server

    Ok(())
}

/// used to start a synthesiser, should be called only once
async fn start() -> Result<()> {
    // TODO: write start routine

    // TODO: turn ras-pi LED on & yellow

    // start command and control (c2) server (w/ registration url & serial coms w/ micro-controller)
    controller::start("127.0.0.1").await?;
    // turn ras-pi LED orange
    // read config file
    // start the default set of module containers
    // turn ras-pi LED red

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
