use std::sync::Arc;

use axum::extract::State;
use services::indexer::{FetchListPayload, IndexerClient};

use crate::auth::api_client::AdminWrite;
use crate::error::ApiError;
use crate::request::Json;
use crate::response::ApiResponse;

pub async fn add_list(_permission: AdminWrite, State(client): State<Arc<IndexerClient>>, Json(payload): Json<FetchListPayload>) -> Result<ApiResponse<FetchListPayload>, ApiError> {
    client.fetch_list(payload.clone()).await?;
    Ok(payload.into())
}
