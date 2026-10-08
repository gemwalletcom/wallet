use std::sync::Arc;

use axum::extract::State;
use primitives::{AddressDetails, AddressName, ChainAddress};
use services::transactions::{AddressDetailsClient, AddressNamesClient};

use crate::auth::device::{AuthenticatedDevice, DeviceJson};
use crate::error::ApiError;
use crate::request::{AddressParam, ChainParam, Path};
use crate::response::ApiResponse;

pub async fn get_address_names(_device: AuthenticatedDevice, State(client): State<Arc<AddressNamesClient>>, requests: DeviceJson<Vec<ChainAddress>>) -> Result<ApiResponse<Vec<AddressName>>, ApiError> {
    Ok(client.get_address_names(requests.into_inner()).await?.into())
}

pub async fn get_address_details(_device: AuthenticatedDevice, Path((chain, address)): Path<(ChainParam, AddressParam)>, State(client): State<Arc<AddressDetailsClient>>) -> Result<ApiResponse<AddressDetails>, ApiError> {
    Ok(client.get_address_details(ChainAddress::new(chain.0, address.0)).await?.into())
}
