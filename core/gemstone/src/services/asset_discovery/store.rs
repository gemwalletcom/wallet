use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{AssetId, WalletId};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemAssetDiscoveryStore: Send + Sync {
    async fn get_assets_timestamp(&self, wallet_id: WalletId) -> Result<u64, GemServiceError>;
    async fn set_assets_timestamp(&self, wallet_id: WalletId, timestamp: u64) -> Result<(), GemServiceError>;
    async fn enable_assets(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError>;
}
