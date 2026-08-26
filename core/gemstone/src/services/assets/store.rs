use async_trait::async_trait;
use primitives::WalletId;
use primitives::{Asset, AssetBasic, AssetFull, AssetId};

use super::error::GemAssetError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemAssetStore: Send + Sync {
    async fn get_asset_ids(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemAssetError>;
    async fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemAssetError>;
    async fn save_assets(&self, assets: Vec<AssetBasic>) -> Result<(), GemAssetError>;
    async fn save_asset(&self, asset: AssetFull) -> Result<(), GemAssetError>;
    async fn add_missing_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemAssetError>;
}
