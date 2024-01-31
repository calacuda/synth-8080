use crate::{
    common::{mk_float, not_found},
    Float,
};
use actix::{Actor, StreamHandler};
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use anyhow::{ensure, Result};
use rodio::{OutputStream, Source};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

pub const N_INPUTS: u8 = 1;

// #[derive(Clone)]
// struct Buff {
//     buff: [Float; 512],
//     i: usize,
// }
//
// impl Buff {
//     fn new() -> Self {
//         Self {
//             buff: [0.0; 512],
//             i: 0,
//         }
//     }
//
//     fn insert(&mut self, n: Float) {
//         self.i = (self.i + 1) % 512;
//         self.buff[self.i] = n;
//     }
//
//     fn get(&self) -> Float {
//         self.buff[self.i]
//     }
// }

#[derive(Clone)]
struct Audio(Arc<Mutex<Float>>);

impl Audio {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(0.0)))
    }
}

impl Iterator for Audio {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // let sample = (*self.0.lock().unwrap()) as f32;
        // info!("{}", self.0.lock().unwrap().i);
        Some((*self.0.lock().unwrap()) as f32)
    }
}

impl Source for Audio {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        48_000
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

struct InputCtrl {
    state: web::Data<Audio>,
}

impl InputCtrl {
    pub fn new(i: u8, state: web::Data<Audio>) -> Result<Self> {
        ensure!(i < N_INPUTS, "Output module only has one input");
        Ok(Self { state })
    }
}

impl Actor for InputCtrl {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for InputCtrl {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {
        match msg {
            // Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            // Ok(ws::Message::Text(text)) => ctx.text(format!("echoing {text}")),
            Ok(ws::Message::Binary(bin)) => {
                // ctx.binary(bin)
                // info!("{bin:?}");
                if let Ok(sample) = mk_float(&*bin) {
                    // info!("sample = {sample}");
                    let mut buf = (*self.state).0.lock().unwrap();
                    *buf = sample;
                    // let i = state.i;
                    // (*state).buff[i] = sample;
                } else {
                    error!("failed to convert binary message to a Float.");
                }
                // ctx.stop();
            }
            _ => (),
        }
    }
}

#[get("/input/{i}")]
async fn module_input(
    data: web::Data<Audio>,
    path: web::Path<u8>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let input_n = *path;
    info!("input {input_n}");

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

pub async fn start(ip: &str, port: u16) -> anyhow::Result<()> {
    let audio = Audio::new();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    stream_handle.play_raw(audio.clone())?;

    let shared = web::Data::new(audio);

    HttpServer::new(move || App::new().app_data(shared.clone()).service(module_input))
        .bind((ip, port))?
        .run()
        .await?;

    Ok(())
}
