use std::sync::Arc;

use axum::extract::State;
use primitives::{AssetId, DefiPosition};
use services::assets::AssetsClient;
use services::defi::DefiClient;

use crate::auth::device::AuthenticatedDeviceWallet;
use crate::error::ApiError;
use crate::request::{FromTimestampQuery, Query};
use crate::response::ApiResponse;

pub async fn get_assets(device: AuthenticatedDeviceWallet, Query(query): Query<FromTimestampQuery>, State(client): State<Arc<AssetsClient>>) -> Result<ApiResponse<Vec<AssetId>>, ApiError> {
    Ok(client.get_assets_by_wallet_id(device.record.id, device.wallet_id, query.from_timestamp).await?.into())
}

pub async fn get_defi_positions(device: AuthenticatedDeviceWallet, State(client): State<Arc<DefiClient>>) -> Result<ApiResponse<Vec<DefiPosition>>, ApiError> {
    Ok(client.get_positions_by_wallet_id(device.record.id, device.wallet_id).await?.into())
}
