use std::sync::Arc;

use axum::extract::State;
use primitives::{WalletSubscription, WalletSubscriptionChains};
use services::devices::WalletsClient;

use crate::auth::device::{AuthenticatedDevice, DeviceJson};
use crate::error::ApiError;
use crate::response::ApiResponse;

pub async fn get_subscriptions(device: AuthenticatedDevice, State(client): State<Arc<WalletsClient>>) -> Result<ApiResponse<Vec<WalletSubscriptionChains>>, ApiError> {
    Ok(client.get_subscriptions(device.record.id).await?.into())
}

pub async fn add_subscriptions(device: AuthenticatedDevice, State(client): State<Arc<WalletsClient>>, subscriptions: DeviceJson<Vec<WalletSubscription>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.add_subscriptions(device.record.id, subscriptions.into_inner()).await?.into())
}

pub async fn delete_subscriptions(device: AuthenticatedDevice, State(client): State<Arc<WalletsClient>>, subscriptions: DeviceJson<Vec<WalletSubscriptionChains>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.delete_subscriptions(device.record.id, subscriptions.into_inner()).await?.into())
}
