use std::sync::Arc;

use axum::extract::State;
use primitives::WalletConfigurationResult;
use services::devices::WalletConfigurationClient;

use crate::auth::device::AuthenticatedDeviceWallet;
use crate::error::ApiError;
use crate::response::ApiResponse;

pub async fn get_wallet_configuration(device: AuthenticatedDeviceWallet, State(client): State<Arc<WalletConfigurationClient>>) -> Result<ApiResponse<WalletConfigurationResult>, ApiError> {
    Ok(client.get_configuration(device.record.id, device.wallet_id, device.wallet_identifier, device.wallet_type).await?.into())
}
