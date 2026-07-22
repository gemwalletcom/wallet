use crate::responders::{ApiError, ApiResponse};
use pricer::MarketsClient;
use primitives::Markets;
use rocket::{State, get};

#[get("/markets")]
pub async fn get_markets(client: &State<MarketsClient>) -> Result<ApiResponse<Markets>, ApiError> {
    Ok(client.get_markets().await?.into())
}
