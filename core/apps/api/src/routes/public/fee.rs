use std::sync::Arc;

use axum::extract::State;
use primitives::ChainFeeEstimates;
use services::chain::FeeEstimatesClient;

use crate::error::ApiError;
use crate::response::ApiResponse;

pub async fn get_fee_estimates(State(client): State<Arc<FeeEstimatesClient>>) -> Result<ApiResponse<Vec<ChainFeeEstimates>>, ApiError> {
    Ok(client.get_fee_estimates().await?.into())
}
