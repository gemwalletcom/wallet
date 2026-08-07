pub mod client;
use crate::responders::{ApiError, ApiResponse};
pub use client::ConfigClient;
use primitives::config::ConfigResponse;
use rocket::{State, get};

#[get("/config")]
pub async fn get_config(config_client: &State<ConfigClient>) -> Result<ApiResponse<ConfigResponse>, ApiError> {
    Ok(config_client.get_config()?.into())
}
