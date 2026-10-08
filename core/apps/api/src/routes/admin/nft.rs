use std::sync::Arc;

use axum::extract::State;
use services::indexer::IndexerClient;
use services::nft::NFTClient;

use crate::auth::api_client::AdminWrite;
use crate::error::ApiError;
use crate::request::{NftAssetIdParam, NftCollectionIdParam, Path};
use crate::response::ApiResponse;

pub async fn update_nft_collection(_permission: AdminWrite, Path(collection_id): Path<NftCollectionIdParam>, State(client): State<Arc<NFTClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.update_collection(collection_id.0).await?.into())
}

pub async fn update_nft_asset(_permission: AdminWrite, Path(asset_id): Path<NftAssetIdParam>, State(client): State<Arc<IndexerClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.refresh_nft_asset(asset_id.0).await?.into())
}
