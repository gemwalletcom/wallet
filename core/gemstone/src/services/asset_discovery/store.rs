use async_trait::async_trait;
use primitives::{AssetId, WalletId};

use super::error::GemAssetDiscoveryError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemAssetDiscoveryStore: Send + Sync {
    async fn get_assets_timestamp(&self, wallet_id: WalletId) -> Result<u64, GemAssetDiscoveryError>;
    async fn set_assets_timestamp(&self, wallet_id: WalletId, timestamp: u64) -> Result<(), GemAssetDiscoveryError>;
    async fn enable_assets(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemAssetDiscoveryError>;
}
