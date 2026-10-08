use std::sync::Arc;

use axum::extract::State;
use gem_auth::create_device_token;
use primitives::{AuthNonce, DeviceToken};
use services::auth::AuthClient;

use crate::auth::device::{AuthenticatedDevice, DeviceAuthConfig};
use crate::error::ApiError;
use crate::response::ApiResponse;

pub async fn get_nonce(device: AuthenticatedDevice, State(client): State<Arc<AuthClient>>) -> Result<ApiResponse<AuthNonce>, ApiError> {
    Ok(client.get_nonce(&device.record.device.id).await?.into())
}

pub async fn get_token(device: AuthenticatedDevice, State(config): State<Arc<DeviceAuthConfig>>) -> Result<ApiResponse<DeviceToken>, ApiError> {
    Ok(create_device_token(&device.record.device.id, &config.jwt.secret, config.jwt.expiry)?.into())
}
