use rocket::{State, get};
use services::chain::{ChainFeeEstimates, FeeEstimatesClient};

use crate::api_clients::PermissionChainRead;
use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};

#[get("/chain/fee-estimates/<chain>")]
pub async fn get_chain_fee_estimates(_permission: PermissionChainRead, chain: ChainParam, client: &State<FeeEstimatesClient>) -> Result<ApiResponse<ChainFeeEstimates>, ApiError> {
    Ok(client.get_chain_fee_estimates(chain.0).await?.into())
}

#[get("/chain/fee-estimates")]
pub async fn get_fee_estimates(client: &State<FeeEstimatesClient>) -> Result<ApiResponse<Vec<ChainFeeEstimates>>, ApiError> {
    Ok(client.get_fee_estimates().await?.into())
}
