use crate::{router::Router, Float, FLOAT_LEN};
use actix_web::{http::StatusCode, Error, HttpResponse};
use anyhow::{ensure, Result};
use std::{
    ops::{Deref, Index},
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

pub mod notes;

#[derive(Clone, Copy)]
pub struct Connection {
    pub idx: usize,
    pub src_output: u8,
    pub dest_module: u8,
    pub dest_input: u8,
}

impl Index<Connection> for Router {
    type Output = Arc<Mutex<Vec<Option<Float>>>>;

    fn index(&self, index: Connection) -> &Self::Output {
        &self.deref()[index.dest_module as usize][index.dest_input as usize]
    }
}

// impl IndexMut<Connection> for Router {
//     // type Output = Option<Float>;
//
//     fn index_mut(&mut self, index: Connection) -> &mut Self::Output {
//         &mut self[index]
//     }
// }

pub trait Module {
    fn start(&self) -> anyhow::Result<JoinHandle<()>>;
    fn connect(&self, output: u8, module_id: u8, input: u8);
}

pub async fn not_found() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::build(StatusCode::NOT_FOUND)
        .content_type("text/html; charset=utf-8")
        .body("<h1>Error 404</h1>"))
}

pub fn mk_float(b: &[u8]) -> Result<Float> {
    ensure!(b.len() == FLOAT_LEN, "length of bytes bust be ");

    Ok(Float::from_le_bytes([
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
    ]))
}

pub fn bend_range() -> Float {
    (2.0 as Float).powf(2.0 / 12.0)
}
