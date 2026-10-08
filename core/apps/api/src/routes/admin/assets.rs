use std::sync::Arc;

use axum::extract::State;
use primitives::AssetId;
use services::indexer::{FetchAssetAssociationsPayload, IndexerClient};

use crate::auth::api_client::AdminWrite;
use crate::error::ApiError;
use crate::request::Json;
use crate::response::ApiResponse;

pub async fn add_asset(_permission: AdminWrite, State(client): State<Arc<IndexerClient>>, Json(asset_id): Json<AssetId>) -> Result<ApiResponse<AssetId>, ApiError> {
    client.refresh_asset(asset_id.clone()).await?;
    Ok(asset_id.into())
}

pub async fn add_asset_associations(_permission: AdminWrite, State(client): State<Arc<IndexerClient>>, Json(payload): Json<FetchAssetAssociationsPayload>) -> Result<ApiResponse<FetchAssetAssociationsPayload>, ApiError> {
    client.fetch_asset_associations(payload.clone()).await?;
    Ok(payload.into())
}

pub async fn get_asset_status(_permission: AdminWrite, State(client): State<Arc<IndexerClient>>, Json(asset_id): Json<AssetId>) -> Result<ApiResponse<AssetId>, ApiError> {
    if asset_id.is_native() {
        return Err(ApiError::BadRequest("Asset status requires a token asset".to_string()));
    }
    client.fetch_asset_status(asset_id.clone()).await?;
    Ok(asset_id.into())
}
