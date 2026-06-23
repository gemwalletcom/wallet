use rocket::{State, get, tokio::sync::Mutex};

use crate::api_clients::PermissionChainRead;
use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::StakeValidator;

use super::ChainClient;

#[get("/chain/staking/<chain>/validators")]
pub async fn get_validators(_permission: PermissionChainRead, chain: ChainParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<Vec<StakeValidator>>, ApiError> {
    Ok(client.lock().await.get_validators(chain.0).await?.into())
}

#[get("/chain/staking/<chain>/apy")]
pub async fn get_staking_apy(_permission: PermissionChainRead, chain: ChainParam, client: &State<Mutex<ChainClient>>) -> Result<ApiResponse<f64>, ApiError> {
    Ok(client.lock().await.get_staking_apy(chain.0).await?.into())
}
