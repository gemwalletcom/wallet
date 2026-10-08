use std::sync::Arc;

use axum::extract::State;
use primitives::nft::NFTAssetData;
use primitives::{AssetId, NFTData, ReportNft};
use services::indexer::IndexerClient;
use services::nft::NFTClient;

use crate::auth::device::{AuthenticatedDevice, AuthenticatedDeviceWallet, DeviceJson};
use crate::error::ApiError;
use crate::request::{NftAssetIdParam, Path};
use crate::response::ApiResponse;

pub async fn get_nft_assets(device: AuthenticatedDeviceWallet, State(client): State<Arc<NFTClient>>) -> Result<ApiResponse<Vec<NFTData>>, ApiError> {
    Ok(client.get_wallet_assets(device.record.id, device.wallet_id).await?.into())
}

pub async fn get_nft_asset(_device: AuthenticatedDevice, Path(asset_id): Path<NftAssetIdParam>, State(client): State<Arc<NFTClient>>) -> Result<ApiResponse<NFTAssetData>, ApiError> {
    Ok(client.get_nft_asset_data(asset_id.0).await?.into())
}

pub async fn refresh_nft_asset(_device: AuthenticatedDeviceWallet, Path(asset_id): Path<NftAssetIdParam>, State(client): State<Arc<IndexerClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.fetch_nft_asset(asset_id.0).await?.into())
}

pub async fn report_nft(device: AuthenticatedDevice, State(client): State<Arc<NFTClient>>, request: DeviceJson<ReportNft>) -> Result<ApiResponse<bool>, ApiError> {
    let request = request.into_inner();
    let asset_id = request
        .asset_id
        .as_deref()
        .map(|asset_id| AssetId::new(asset_id).ok_or_else(|| ApiError::bad_request(format!("Invalid asset_id: {asset_id}"))))
        .transpose()?;
    Ok(client.report_nft(&device.record.device.id, request.collection_id.clone(), asset_id, request.reason.clone()).await?.into())
}
