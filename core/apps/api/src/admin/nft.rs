use rocket::{State, put};
use services::indexer::IndexerClient;
use services::nft::NFTClient;

use crate::api_clients::PermissionAdminWrite;
use crate::params::{NftAssetIdParam, NftCollectionIdParam};
use crate::responders::{ApiError, ApiResponse};

#[put("/nft/collections/update/<collection_id>")]
pub async fn update_nft_collection(_permission: PermissionAdminWrite, collection_id: NftCollectionIdParam, client: &State<NFTClient>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.update_collection(collection_id.0).await?.into())
}

#[put("/nft/assets/update/<asset_id>")]
pub async fn update_nft_asset(_permission: PermissionAdminWrite, asset_id: NftAssetIdParam, client: &State<IndexerClient>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.refresh_nft_asset(asset_id.0).await?.into())
}
