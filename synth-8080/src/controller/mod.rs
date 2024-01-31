use crate::NodeType;
use actix_web::{get, post, web, App, HttpServer};
use serde::{Deserialize, Serialize};
use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};
use tracing::info;

struct Connection {
    /// ip address of the source module
    pub src: IpAddr,
    /// ip address of the destination module
    pub dest: IpAddr,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Module {
    /// the modules ip address
    pub ip: IpAddr,
    /// the port number that the module is listening on
    pub port: u16,
    // /// the nunmber of inputs
    // pub n_ins: u8,
    // /// the number of outputs
    // pub n_outs: u8,
    /// the type of module
    pub kind: NodeType,
}

struct Controller {
    /// the liist of connections
    pub connections: Arc<Mutex<Vec<Connection>>>,
    /// the list of registered modules
    pub modules: Arc<Mutex<Vec<Module>>>,
    // TODO: find a serial library to talk to the micro controller
    // /// UART connectino to the micro-controller
    // pub serial: Arc<Mutex<>>,
}

impl Controller {
    pub fn new() -> Self {
        let connections = Arc::new(Mutex::new(Vec::new()));
        let modules = Arc::new(Mutex::new(Vec::new()));

        Self {
            connections,
            modules,
        }
    }
}

#[post("/register/")]
async fn register(data: web::Data<Controller>, module: web::Json<Module>) -> String {
    data.modules.lock().unwrap().push((*module).clone());
    info!("registered {}:{}", module.ip, module.port);
    "success".to_string()
}

#[get("/connect/{src_ip}/{output}/{dest_ip}/{input}")]
async fn connect_to(
    data: web::Data<Controller>,
    path: web::Path<(IpAddr, u8, IpAddr, u8)>,
) -> String {
    if let Ok(res) = reqwest::get(format!(
        "http://{}/conntect/{}/{}/{}",
        path.0, path.1, path.2, path.3
    ))
    .await
    {
        let code = res.status();
        info!("connect command http code {code}");
        "Success".to_string()
    } else {
        "failed".to_string()
    }
}

pub async fn start(ip: &str) -> anyhow::Result<()> {
    let ctrlr = web::Data::new(Controller::new());

    HttpServer::new(move || {
        App::new()
            .app_data(ctrlr.clone())
            .service(register)
            .service(connect_to)
    })
    .bind((ip, 8080))?
    .run()
    .await?;

    Ok(())
}
