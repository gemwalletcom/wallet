use std::sync::Arc;

use axum::extract::State;
use primitives::ChainAddress;
use services::indexer::IndexerClient;

use crate::auth::api_client::AdminWrite;
use crate::error::ApiError;
use crate::request::Json;
use crate::response::ApiResponse;

pub async fn refresh_addresses(_permission: AdminWrite, State(client): State<Arc<IndexerClient>>, Json(addresses): Json<Vec<ChainAddress>>) -> Result<ApiResponse<Vec<ChainAddress>>, ApiError> {
    client.refresh_addresses(&addresses).await?;
    Ok(addresses.into())
}
