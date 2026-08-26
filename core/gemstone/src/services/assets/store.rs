use async_trait::async_trait;
use primitives::{Asset, AssetBasic, AssetId};

use super::error::GemAssetError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemAssetStore: Send + Sync {
    async fn get_asset_ids(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemAssetError>;
    async fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemAssetError>;
    async fn add_assets(&self, assets: Vec<AssetBasic>) -> Result<(), GemAssetError>;
    async fn add_missing_balances(&self, wallet_id: String, asset_ids: Vec<AssetId>) -> Result<(), GemAssetError>;
}
