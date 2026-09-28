use primitives::ChainAddress;
use rocket::serde::json::Json;
use rocket::{State, post};
use services::indexer::IndexerClient;

use crate::api_clients::PermissionAdminWrite;
use crate::responders::{ApiError, ApiResponse};

#[post("/addresses/refresh", format = "json", data = "<addresses>")]
pub async fn refresh_addresses(_permission: PermissionAdminWrite, addresses: Json<Vec<ChainAddress>>, client: &State<IndexerClient>) -> Result<ApiResponse<Vec<ChainAddress>>, ApiError> {
    let addresses = addresses.into_inner();
    client.refresh_addresses(&addresses).await?;
    Ok(addresses.into())
}
