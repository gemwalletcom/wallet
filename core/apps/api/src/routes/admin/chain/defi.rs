use std::sync::Arc;

use axum::extract::State;
use defi::DefiProviderClient;
use primitives::DefiPosition;

use crate::auth::api_client::ChainRead;
use crate::error::ApiError;
use crate::request::{AddressParam, ChainParam, Path};
use crate::response::ApiResponse;

pub async fn get_defi_positions(_permission: ChainRead, Path((chain, address)): Path<(ChainParam, AddressParam)>, State(client): State<Arc<DefiProviderClient>>) -> Result<ApiResponse<Vec<DefiPosition>>, ApiError> {
    Ok(client.get_positions(chain.0, &address.0).await?.into())
}
