use std::sync::Arc;

use axum::extract::State;
use primitives::Asset;
use services::chain::ChainClient;

use crate::auth::api_client::ChainRead;
use crate::error::ApiError;
use crate::request::{ChainParam, Path};
use crate::response::ApiResponse;

pub async fn get_token(_permission: ChainRead, Path((chain, token_id)): Path<(ChainParam, String)>, State(client): State<Arc<ChainClient>>) -> Result<ApiResponse<Asset>, ApiError> {
    Ok(client.get_token_data(chain.0, token_id).await?.into())
}
