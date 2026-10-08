use std::sync::Arc;

use axum::extract::State;
use services::indexer::{FetchPricesPayload, IndexerClient};

use crate::auth::api_client::AdminWrite;
use crate::error::ApiError;
use crate::request::Json;
use crate::response::ApiResponse;

pub async fn add_price(_permission: AdminWrite, State(client): State<Arc<IndexerClient>>, Json(payload): Json<FetchPricesPayload>) -> Result<ApiResponse<FetchPricesPayload>, ApiError> {
    client.fetch_prices(payload.clone()).await?;
    Ok(payload.into())
}
