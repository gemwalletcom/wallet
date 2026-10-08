use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use primitives::NFTResource;
use services::nft::NFTClient;

use crate::error::ApiError;
use crate::request::{NftAssetIdParam, NftCollectionIdParam, Path};

pub async fn get_nft_asset_preview(Path(asset_id): Path<NftAssetIdParam>, State(client): State<Arc<NFTClient>>) -> Result<Json<NFTResource>, ApiError> {
    Ok(Json(client.load_nft_asset(&asset_id.0.to_string()).await?.images.preview))
}

pub async fn get_nft_asset_resource(Path(asset_id): Path<NftAssetIdParam>, State(client): State<Arc<NFTClient>>) -> Result<Json<NFTResource>, ApiError> {
    Ok(Json(client.load_nft_asset(&asset_id.0.to_string()).await?.resource))
}

pub async fn get_nft_collection_preview(Path(collection_id): Path<NftCollectionIdParam>, State(client): State<Arc<NFTClient>>) -> Result<Json<NFTResource>, ApiError> {
    Ok(Json(client.load_nft_collection(&collection_id.0.to_string()).await?.images.preview))
}
