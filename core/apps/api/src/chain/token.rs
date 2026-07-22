use rocket::{State, get};

use crate::api_clients::PermissionChainRead;
use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::Asset;

use super::ChainClient;

#[get("/chain/token/<chain>/<token_id>/info")]
pub async fn get_token(_permission: PermissionChainRead, chain: ChainParam, token_id: &str, client: &State<ChainClient>) -> Result<ApiResponse<Asset>, ApiError> {
    Ok(client.get_token_data(chain.0, token_id.to_string()).await?.into())
}
