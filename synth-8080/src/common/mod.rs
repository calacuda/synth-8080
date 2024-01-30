use crate::{Float, FLOAT_LEN};
use actix_web::{http::StatusCode, Error, HttpResponse};
use anyhow::{ensure, Result};

pub mod notes;

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
