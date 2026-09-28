use rocket::{State, post, serde::json::Json};
use services::indexer::{FetchPricesPayload, IndexerClient};

use crate::api_clients::PermissionAdminWrite;
use crate::responders::{ApiError, ApiResponse};

#[post("/prices/add", format = "json", data = "<payload>")]
pub async fn add_price(_permission: PermissionAdminWrite, payload: Json<FetchPricesPayload>, client: &State<IndexerClient>) -> Result<ApiResponse<FetchPricesPayload>, ApiError> {
    let payload = payload.into_inner();
    client.fetch_prices(payload.clone()).await?;
    Ok(payload.into())
}
