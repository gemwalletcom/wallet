use std::sync::Arc;

use axum::extract::State;
use primitives::StakeValidator;
use services::chain::ChainClient;

use crate::auth::api_client::ChainRead;
use crate::error::ApiError;
use crate::request::{ChainParam, Path};
use crate::response::ApiResponse;

pub async fn get_validators(_permission: ChainRead, Path(chain): Path<ChainParam>, State(client): State<Arc<ChainClient>>) -> Result<ApiResponse<Vec<StakeValidator>>, ApiError> {
    Ok(client.get_validators(chain.0).await?.into())
}

pub async fn get_staking_apy(_permission: ChainRead, Path(chain): Path<ChainParam>, State(client): State<Arc<ChainClient>>) -> Result<ApiResponse<f64>, ApiError> {
    Ok(client.get_staking_apy(chain.0).await?.into())
}
