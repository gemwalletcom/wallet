use cacher::{CacheKey, CacherClient};
use primitives::AssetId;
use rocket::{State, post, serde::json::Json};
use storage::{AssetsRepository, Database};
use streamer::{FetchAssetAssociationsPayload, StreamProducer, StreamProducerQueue};

use crate::api_clients::PermissionAdminWrite;
use crate::responders::{ApiError, ApiResponse};

#[post("/assets/add", format = "json", data = "<asset_id>")]
pub async fn add_asset(
    _permission: PermissionAdminWrite,
    asset_id: Json<AssetId>,
    cacher: &State<CacherClient>,
    stream_producer: &State<StreamProducer>,
) -> Result<ApiResponse<AssetId>, ApiError> {
    let asset_id = asset_id.into_inner();
    cacher.delete(&CacheKey::FetchAssets(&asset_id.to_string()).key()).await?;
    stream_producer.publish_fetch_assets(vec![asset_id.clone()]).await?;
    Ok(asset_id.into())
}

#[post("/assets/associations/add", format = "json", data = "<payload>")]
pub async fn add_asset_associations(
    _permission: PermissionAdminWrite,
    payload: Json<FetchAssetAssociationsPayload>,
    stream_producer: &State<StreamProducer>,
) -> Result<ApiResponse<FetchAssetAssociationsPayload>, ApiError> {
    let payload = payload.into_inner();
    stream_producer.publish_fetch_asset_associations(payload.clone()).await?;
    Ok(payload.into())
}

#[post("/assets/status", format = "json", data = "<asset_id>")]
pub async fn fetch_asset_status(
    _permission: PermissionAdminWrite,
    asset_id: Json<AssetId>,
    database: &State<Database>,
    stream_producer: &State<StreamProducer>,
) -> Result<ApiResponse<AssetId>, ApiError> {
    let asset_id = asset_id.into_inner();
    if asset_id.is_native() {
        return Err(ApiError::BadRequest("Asset status requires a token asset".to_string()));
    }
    database.assets()?.get_asset(&asset_id)?;
    stream_producer.publish_fetch_asset_status(asset_id.clone()).await?;
    Ok(asset_id.into())
}
