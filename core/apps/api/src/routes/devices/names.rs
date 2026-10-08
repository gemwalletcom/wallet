use std::sync::Arc;

use axum::extract::State;
use name_resolver::NameClient;
use primitives::name::NameRecord;

use crate::auth::device::AuthenticatedDevice;
use crate::error::ApiError;
use crate::request::{ChainQuery, Path, Query};
use crate::response::ApiResponse;

pub async fn get_name(_device: AuthenticatedDevice, Path(name): Path<String>, Query(query): Query<ChainQuery>, State(client): State<Arc<NameClient>>) -> Result<ApiResponse<Option<NameRecord>>, ApiError> {
    Ok(client.resolve(&name, query.chain.0).await?.into())
}
