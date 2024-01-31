use crate::{
    common::{bend_range, mk_float, not_found, notes::Note},
    osc::{sin_wt::WavetableOscillator, Osc},
    Float,
};
use actix::{Actor, StreamHandler};
use actix_web::{
    get, http::header::ContentType, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws;
use anyhow::{bail, ensure, Result};
use serde::Deserialize;
use std::{
    net::{IpAddr, Ipv4Addr},
    ops::DerefMut,
    str::FromStr,
    sync::{Arc, Mutex},
};
use tokio::{
    task::{spawn, JoinHandle},
    time::{sleep, Duration},
};
use tracing::{error, info};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

pub const N_INPUTS: u8 = 3;
pub const N_OUTPUTS: u8 = 2;

#[derive(Deserialize, Debug, Clone)]
pub enum OscType {
    #[serde(rename = "sine", alias = "sin")]
    Sine,
    #[serde(rename = "square", alias = "squ")]
    Square,
    #[serde(rename = "triangle", alias = "tri")]
    Triangle,
    #[serde(rename = "saw-tooth", alias = "sawtooth", alias = "saw")]
    SawTooth,
}

enum Input {
    Volume = 0,
    PitchBend = 1,
    Pitch = 2,
}

impl TryFrom<u8> for Input {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Volume),
            1 => Ok(Self::PitchBend),
            2 => Ok(Self::Pitch),
            _ => bail!("invalid input number"),
        }
    }
}

struct Vco {
    pub osc_type: Arc<Mutex<OscType>>,
    /// the oscilator that produces samples
    pub osc: Arc<Mutex<Box<dyn Osc>>>,
    /// a web sockets connection to an input that consumes the samples produced by self.osc
    pub pitch_output: Arc<
        Mutex<
            Vec<(
                WebSocket<MaybeTlsStream<std::net::TcpStream>>,
                tungstenite::http::Response<std::option::Option<Vec<u8>>>,
            )>,
        >,
    >,
    /// a gate that control when the note is playing, this is done to allow for polyphony
    pub play_output: Arc<
        Mutex<
            Vec<(
                WebSocket<MaybeTlsStream<std::net::TcpStream>>,
                tungstenite::http::Response<std::option::Option<Vec<u8>>>,
            )>,
        >,
    >,
    /// where the data from the volume input is stored
    pub volume_in: Arc<Mutex<Vec<Float>>>,
    /// where the data from the pitch bend input is stored
    pub pitch_bend_in: Arc<Mutex<Vec<Float>>>,
    /// the note to be played
    pub pitch_in: Arc<Mutex<Vec<Float>>>,
    /// wether the oscilator should produce over tones.
    pub overtones: Arc<Mutex<bool>>,
    /// the thread handle that computes generates the next sample
    pub generator: Arc<Mutex<JoinHandle<()>>>,
    /// the note the oscilator is suposed to make, used to reset self.osc if pitch_bend_in
    /// disconnects
    pub note: Arc<Mutex<Note>>,
    /// how much to bend the pitch when pitch bends happen
    pub bend_amt: Arc<Float>,
}

impl Vco {
    pub fn new() -> Self {
        let osc_type = Arc::new(Mutex::new(OscType::Sine));
        let osc: Arc<Mutex<Box<dyn Osc>>> =
            Arc::new(Mutex::new(Box::new(WavetableOscillator::new())));
        let pitch_output = Arc::new(Mutex::new(Vec::new()));
        let play_output = Arc::new(Mutex::new(Vec::new()));
        let volume_in = Arc::new(Mutex::new(Vec::new()));
        let pitch_bend_in = Arc::new(Mutex::new(Vec::new()));
        let pitch_in = Arc::new(Mutex::new(Vec::new()));
        let overtones = Arc::new(Mutex::new(false));
        let generator = Arc::new(Mutex::new(spawn(async {})));
        let note = Arc::new(Mutex::new(Note::A4));
        let bend_amt = Arc::new(bend_range());

        Self {
            osc_type,
            osc,
            pitch_output,
            play_output,
            volume_in,
            pitch_bend_in,
            pitch_in,
            overtones,
            generator,
            note,
            bend_amt,
        }
    }

    pub async fn connect_to(&self, output_select: u8, ip: IpAddr, input_select: u8) -> Result<()> {
        ensure!(output_select < N_OUTPUTS, "invalid output selection");

        let uri = Url::from_str(format!("ws://{}:8080/input/{}", ip, input_select).as_str())?;
        let connection = connect(uri)?;

        if output_select == 0 {
            self.pitch_output.lock().unwrap().push(connection);
        } else if output_select == 1 {
            self.play_output.lock().unwrap().push(connection);
        } else {
            bail!("unhandled valid output selction. in other words a valid output was selected but that output handling code waas not yet written.");
        }

        Ok(())
    }

    pub fn set_osc_type(&self, osc_type: OscType) -> String {
        let mut ot = self.osc_type.lock().unwrap();
        *ot = osc_type;

        match *ot {
            OscType::Sine => {
                let mut osc = self.osc.lock().unwrap();
                *osc = Box::new(WavetableOscillator::new());
                "set to sine wave".to_string()
            }
            OscType::Square => "not implemented yet".to_string(),
            OscType::Triangle => "not implemented yet".to_string(),
            OscType::SawTooth => "not implemented yet".to_string(),
        }
    }

    pub fn set_overtones(&self, on: bool) -> String {
        let mut ovt = self.overtones.lock().unwrap();
        *ovt = on;

        // TODO: make twang oscilator for over-tones

        format!("overtones on: {on}")
    }

    /// starts a thread to generate samples.
    pub fn start(&self) {
        let osc = self.osc.clone();
        let outs = self.pitch_output.clone();

        let jh = spawn(async move {
            // let mut last_send = Instant::now();

            loop {
                sleep(Duration::from_nanos(20830)).await;
                let sample = osc.lock().unwrap().get_sample().to_le_bytes();
                outs.lock()
                    .unwrap()
                    .deref_mut()
                    .iter_mut()
                    .for_each(|(socket, _res)| {
                        if let Err(e) = socket.send(Message::Binary(sample.to_vec())) {
                            error!("{e}")
                        }
                    });
            }
        });

        let mut gen = self.generator.lock().unwrap();
        *gen = jh;
    }

    /// applies a pitch bend by changing the oscilators frequency
    fn apply_bend(&self, bend: Float) {
        let note = self.note.lock().unwrap();
        let note: Float = (*note).clone().into();

        // get frequency shift
        // self.note + (bend * )
        let shift = bend * (note * *self.bend_amt);

        let mut osc = self.osc.lock().unwrap();
        (*osc).set_frequency(if bend > 0.0 {
            note + (note * shift)
        } else {
            note - (note / shift)
        });
    }

    pub fn set_note(&self, note: Note) -> String {
        let mut n = self.note.lock().unwrap();
        // get frequency from note
        *n = note;

        format!("set note to {n}")
    }
}

/// Define HTTP actor
struct InputCtrl {
    input: Input,
    state: web::Data<Vco>,
}

impl InputCtrl {
    pub fn new(input_n: u8, state: web::Data<Vco>) -> Result<Self> {
        Ok(Self {
            input: Input::try_from(input_n)?,
            state,
        })
    }
}

impl Actor for InputCtrl {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for InputCtrl {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(format!("echoing {text}")),
            Ok(ws::Message::Binary(bin)) => {
                // ctx.binary(bin)
                if let Ok(sample) = mk_float(&*bin) {
                    match self.input {
                        Input::Volume => self.state.volume_in.lock().unwrap().push(sample),
                        Input::Pitch => self.state.pitch_in.lock().unwrap().push(sample),
                        Input::PitchBend => self.state.pitch_bend_in.lock().unwrap().push(sample),
                    }
                } else {
                    error!("failed to convert binary message to a Float.");
                }
            }
            _ => (),
        }
    }
}

#[get("/connect/{output}/{ip}/{input}")]
async fn connect_to(data: web::Data<Vco>, path: web::Path<(u8, IpAddr, u8)>) -> String {
    // info!("connect command recieved");
    if let Err(e) = data.connect_to(path.0, path.1, path.2).await {
        let res = format!("failed to connect to output, got error: {e}");
        error!(res);

        res
    } else {
        format!(
            "connecting output {} to input ws://{}:8080/{}",
            path.0, path.1, path.2
        )
    }
}

#[get("/set/osc/{type}")]
async fn set_osc_type(data: web::Data<Vco>, path: web::Path<OscType>) -> String {
    info!("setting oscilator type to {:?}", path.clone());
    data.set_osc_type(path.clone())
}

#[get("/set/overtones/{on}")]
async fn set_overtones(data: web::Data<Vco>, path: web::Path<bool>) -> String {
    info!("turning overtones {:?}", path);
    data.set_overtones(*path)
}

#[get("/set/note/{note}")]
async fn set_note(data: web::Data<Vco>, path: web::Path<Note>) -> String {
    info!("setting oscilator note to \"{:?}\"", path.clone());
    data.set_note(path.clone())
}

#[get("/input/{i}")]
async fn module_input(
    data: web::Data<Vco>,
    path: web::Path<u8>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let input_n = *path;

    // if input_n >= N_INPUTS {
    //     return not_found().await;
    // }

    let Ok(ctrl) = InputCtrl::new(input_n, data) else {
        return not_found().await;
    };

    let resp = ws::start(ctrl, &req, stream);
    info!("{:?}", resp);
    resp
}

// pub async fn register(args: &crate::NodeArgs) -> Result<()> {
//     // TODO:send api request to register this module and its info with the c2 module
//
//     Ok(())
// }

pub async fn start(ip: &str, port: u16) -> anyhow::Result<()> {
    let osc = Vco::new();
    osc.start();
    let vco = web::Data::new(osc);

    HttpServer::new(move || {
        App::new()
            .app_data(vco.clone())
            .service(connect_to)
            .service(module_input)
            .service(set_osc_type)
            .service(set_overtones)
    })
    .bind((ip, port))?
    .run()
    .await?;
    Ok(())
}
