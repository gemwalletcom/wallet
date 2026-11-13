use std::sync::Arc;

use axum::extract::State;
use primitives::LightningToken;
use services::lightning::LightningTokenClient;

use crate::error::ApiError;
use crate::response::ApiResponse;

pub async fn get_lightning_token(State(client): State<Arc<LightningTokenClient>>) -> Result<ApiResponse<LightningToken>, ApiError> {
    Ok(client.token().await?.into())
}
