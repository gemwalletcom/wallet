use std::sync::Arc;

use axum::extract::State;
use primitives::ChainFeeEstimates;
use services::chain::FeeEstimatesClient;

use crate::error::ApiError;
use crate::request::{ChainParam, Path};
use crate::response::ApiResponse;

pub async fn get_fee_estimates(State(client): State<Arc<FeeEstimatesClient>>) -> Result<ApiResponse<Vec<ChainFeeEstimates>>, ApiError> {
    Ok(client.get_fee_estimates().await?.into())
}

pub async fn get_chain_fee_estimates(Path(chain): Path<ChainParam>, State(client): State<Arc<FeeEstimatesClient>>) -> Result<ApiResponse<ChainFeeEstimates>, ApiError> {
    Ok(client.get_chain_fee_estimates(chain.0).await?.into())
}
