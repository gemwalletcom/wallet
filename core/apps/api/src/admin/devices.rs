use primitives::{AdminDevice, FiatTransactionData, TransactionsResponse, WalletSubscription};
use rocket::{State, get};

use crate::api_clients::{PermissionDeviceRead, PermissionDeviceSubscriptionsRead, PermissionDeviceTransactionsRead, PermissionFiatTransactionsRead};
use crate::devices::{DevicesClient, FiatQuotesClient, TransactionsClient, WalletsClient};
use crate::responders::{ApiError, ApiResponse};

#[get("/devices/<device_id>")]
pub async fn get_device(
    _permission: PermissionDeviceRead,
    device_id: &str,
    devices: &State<DevicesClient>,
    wallets: &State<WalletsClient>,
) -> Result<ApiResponse<AdminDevice>, ApiError> {
    Ok(devices.get_admin_device(device_id, wallets)?.into())
}

#[get("/devices/<device_id>/subscriptions")]
pub async fn get_device_subscriptions(
    _permission: PermissionDeviceSubscriptionsRead,
    device_id: &str,
    client: &State<WalletsClient>,
) -> Result<ApiResponse<Vec<WalletSubscription>>, ApiError> {
    Ok(client.get_wallet_subscriptions(device_id)?.into())
}

#[get("/devices/<device_id>/wallets/<wallet_id>/subscriptions")]
pub async fn get_device_wallet_subscriptions(
    _permission: PermissionDeviceSubscriptionsRead,
    device_id: &str,
    wallet_id: &str,
    client: &State<WalletsClient>,
) -> Result<ApiResponse<WalletSubscription>, ApiError> {
    Ok(client
        .get_wallet_subscription(device_id, wallet_id)?
        .into())
}

#[get("/devices/<device_id>/transactions")]
pub async fn get_device_transactions(
    _permission: PermissionDeviceTransactionsRead,
    device_id: &str,
    client: &State<TransactionsClient>,
) -> Result<ApiResponse<TransactionsResponse>, ApiError> {
    Ok(client.get_transactions_by_device_id(device_id)?.into())
}

#[get("/devices/<device_id>/fiat/transactions")]
pub async fn get_device_fiat_transactions(
    _permission: PermissionFiatTransactionsRead,
    device_id: &str,
    client: &State<FiatQuotesClient>,
) -> Result<ApiResponse<Vec<FiatTransactionData>>, ApiError> {
    Ok(client.get_transactions_by_device_id(device_id)?.into())
}
