//! Gateway Rest Utils

use http::{Method, Request};
use serde::Deserialize;

use crate::rest::routes::Route;

#[derive(Deserialize, Debug)]
#[allow(missing_docs)]
pub struct GetGateway {
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[allow(missing_docs)]
pub struct GetGatewayBot {
    pub url: String,
    pub shards: u32,
    pub session_start_limit: SessionStartLimit,
}

#[derive(Deserialize, Debug)]
#[allow(missing_docs)]
pub struct SessionStartLimit {
    pub total: u32,
    pub remaining: u32,
    pub reset_after: u32,
    pub max_concurrency: u32,
}

/// Create request for the get gateway endpoint
pub fn get_gateway() -> Request<()> {
    let req = Request::builder()
        .uri(Route::Gateway.to_string())
        .method(Method::GET)
        .body(())
        .unwrap();

    req
}

/// Create request for the get gateway bot endpoint
pub fn get_gateway_bot() -> Request<()> {
    let req = Request::builder()
        .uri(Route::GatewayBot)
        .method(Method::GET)
        .body(())
        .unwrap();

    req
}
