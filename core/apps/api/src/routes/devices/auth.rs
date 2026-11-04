use std::sync::Arc;

use axum::extract::State;
use primitives::AuthNonce;
use services::auth::AuthClient;

use crate::auth::device::AuthenticatedDevice;
use crate::error::ApiError;
use crate::response::ApiResponse;

pub async fn get_nonce(device: AuthenticatedDevice, State(client): State<Arc<AuthClient>>) -> Result<ApiResponse<AuthNonce>, ApiError> {
    Ok(client.get_nonce(&device.record.device.id).await?.into())
}
