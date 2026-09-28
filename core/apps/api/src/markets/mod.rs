use crate::responders::{ApiError, ApiResponse};
use primitives::Markets;
use rocket::{State, get};
use services::prices::MarketsClient;

#[get("/markets")]
pub async fn get_markets(client: &State<MarketsClient>) -> Result<ApiResponse<Markets>, ApiError> {
    Ok(client.get_markets().await?.into())
}
