use std::sync::Arc;

use axum::extract::State;
use primitives::ChainFeeEstimates;
use services::chain::FeeEstimatesClient;

use crate::auth::api_client::ChainRead;
use crate::error::ApiError;
use crate::request::{ChainParam, Path};
use crate::response::ApiResponse;

pub async fn get_chain_fee_estimates(_permission: ChainRead, Path(chain): Path<ChainParam>, State(client): State<Arc<FeeEstimatesClient>>) -> Result<ApiResponse<ChainFeeEstimates>, ApiError> {
    Ok(client.get_chain_fee_estimates(chain.0).await?.into())
}
