use primitives::{Device, TransactionsResponse, WalletSubscription};
use rocket::{State, get, tokio::sync::Mutex};

use crate::api_clients::{PermissionDeviceRead, PermissionDeviceSubscriptionsRead, PermissionDeviceTransactionsRead};
use crate::devices::{DevicesClient, TransactionsClient, WalletsClient};
use crate::responders::{ApiError, ApiResponse};

#[get("/devices/<device_id>")]
pub async fn get_device(_permission: PermissionDeviceRead, device_id: &str, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.get_device(device_id)?.into())
}

#[get("/devices/<device_id>/subscriptions")]
pub async fn get_device_subscriptions(
    _permission: PermissionDeviceSubscriptionsRead,
    device_id: &str,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<Vec<WalletSubscription>>, ApiError> {
    Ok(client.lock().await.get_wallet_subscriptions(device_id)?.into())
}

#[get("/devices/<device_id>/transactions")]
pub async fn get_device_transactions(
    _permission: PermissionDeviceTransactionsRead,
    device_id: &str,
    client: &State<Mutex<TransactionsClient>>,
) -> Result<ApiResponse<TransactionsResponse>, ApiError> {
    Ok(client.lock().await.get_transactions_by_device_id(device_id)?.into())
}
