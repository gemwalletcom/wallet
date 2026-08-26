use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{AssetId, AssetList};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemSearchStore: Send + Sync {
    async fn set_assets(&self, key: String, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError>;
    async fn set_perpetuals(&self, key: String, perpetual_ids: Vec<String>) -> Result<(), GemServiceError>;
    async fn set_lists(&self, key: String, lists: Vec<AssetList>) -> Result<(), GemServiceError>;
}
