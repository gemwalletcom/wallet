use ::nft::NFTClient;
use cacher::{CacheKey, CacherClient};
use rocket::{State, put};
use streamer::{StreamProducer, StreamProducerQueue};

use crate::api_clients::PermissionAdminWrite;
use crate::params::{NftAssetIdParam, NftCollectionIdParam};
use crate::responders::{ApiError, ApiResponse};

#[put("/nft/collections/update/<collection_id>")]
pub async fn update_nft_collection(_permission: PermissionAdminWrite, collection_id: NftCollectionIdParam, client: &State<NFTClient>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.update_collection(&collection_id.0.to_string()).await?.into())
}

#[put("/nft/assets/update/<asset_id>")]
pub async fn update_nft_asset(
    _permission: PermissionAdminWrite,
    asset_id: NftAssetIdParam,
    cacher: &State<CacherClient>,
    stream_producer: &State<StreamProducer>,
) -> Result<ApiResponse<bool>, ApiError> {
    let asset_id = asset_id.0;
    cacher.delete(&CacheKey::FetchNftAsset(&asset_id.to_string()).key()).await?;
    Ok(stream_producer.publish_fetch_nft_asset(asset_id).await?.into())
}
