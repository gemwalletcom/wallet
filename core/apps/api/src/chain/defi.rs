use rocket::{State, get};

use crate::api_clients::PermissionChainRead;
use crate::params::{AddressParam, ChainParam};
use crate::responders::{ApiError, ApiResponse};
use defi::DefiProviderClient;
use primitives::DefiPosition;

#[get("/chain/address/<chain>/<address>/defi/positions")]
pub async fn get_defi_positions(
    _permission: PermissionChainRead,
    chain: ChainParam,
    address: AddressParam,
    client: &State<DefiProviderClient>,
) -> Result<ApiResponse<Vec<DefiPosition>>, ApiError> {
    Ok(client.get_positions(chain.0, &address.0).await?.into())
}
