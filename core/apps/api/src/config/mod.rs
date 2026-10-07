use std::net::IpAddr;

use primitives::config::ConfigResponse;
use rocket::http::Header;
use rocket::response::Responder;
use rocket::{State, get};
use services::app::ConfigClient;

use crate::responders::{ApiError, ApiResponse};

#[get("/config")]
pub async fn get_config(ip: IpAddr, config_client: &State<ConfigClient>) -> Result<ConfigApiResponse, ApiError> {
    Ok(ConfigApiResponse(config_client.get_config(&ip.to_string()).await?.into(), Header::new("Cache-Control", "private, no-store")))
}

#[derive(Responder)]
pub struct ConfigApiResponse(ApiResponse<ConfigResponse>, Header<'static>);
