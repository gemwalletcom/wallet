use std::sync::Arc;

use axum::extract::State;
use primitives::{FiatTransactionData, TransactionsResponse, WalletSubscription};
use services::devices::{AdminDevice, DevicesClient, WalletsClient};
use services::fiat::FiatClient;
use services::transactions::TransactionsClient;

use crate::auth::api_client::{DeviceRead, DeviceSubscriptionsRead, DeviceTransactionsRead, FiatTransactionsRead};
use crate::error::ApiError;
use crate::request::Path;
use crate::response::ApiResponse;

pub async fn get_device(_permission: DeviceRead, Path(device_id): Path<String>, State(devices): State<Arc<DevicesClient>>, State(wallets): State<Arc<WalletsClient>>) -> Result<ApiResponse<AdminDevice>, ApiError> {
    Ok(devices.get_admin_device(&device_id, &wallets).await?.into())
}

pub async fn get_device_subscriptions(_permission: DeviceSubscriptionsRead, Path(device_id): Path<String>, State(client): State<Arc<WalletsClient>>) -> Result<ApiResponse<Vec<WalletSubscription>>, ApiError> {
    Ok(client.get_wallet_subscriptions(&device_id).await?.into())
}

pub async fn get_device_wallet_subscriptions(_permission: DeviceSubscriptionsRead, Path((device_id, wallet_id)): Path<(String, String)>, State(client): State<Arc<WalletsClient>>) -> Result<ApiResponse<WalletSubscription>, ApiError> {
    Ok(client.get_wallet_subscription(&device_id, &wallet_id).await?.into())
}

pub async fn get_device_transactions(_permission: DeviceTransactionsRead, Path(device_id): Path<String>, State(client): State<Arc<TransactionsClient>>) -> Result<ApiResponse<TransactionsResponse>, ApiError> {
    Ok(client.get_transactions_by_device_id(&device_id).await?.into())
}

pub async fn get_device_fiat_transactions(_permission: FiatTransactionsRead, Path(device_id): Path<String>, State(client): State<Arc<FiatClient>>) -> Result<ApiResponse<Vec<FiatTransactionData>>, ApiError> {
    Ok(client.get_transactions_by_device_id(&device_id).await?.into())
}
