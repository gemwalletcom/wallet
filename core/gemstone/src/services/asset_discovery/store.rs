use async_trait::async_trait;
use primitives::AssetId;

use super::error::GemAssetDiscoveryError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemAssetDiscoveryStore: Send + Sync {
    async fn get_assets_timestamp(&self, wallet_id: String) -> Result<u64, GemAssetDiscoveryError>;
    async fn set_assets_timestamp(&self, wallet_id: String, timestamp: u64) -> Result<(), GemAssetDiscoveryError>;
    async fn enable_assets(&self, wallet_id: String, asset_ids: Vec<AssetId>) -> Result<(), GemAssetDiscoveryError>;
}
