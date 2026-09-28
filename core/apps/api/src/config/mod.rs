use crate::responders::{ApiError, ApiResponse};
use primitives::config::ConfigResponse;
use rocket::{State, get};
use services::app::ConfigClient;

#[get("/config")]
pub async fn get_config(config_client: &State<ConfigClient>) -> Result<ApiResponse<ConfigResponse>, ApiError> {
    Ok(config_client.get_config().await?.into())
}
