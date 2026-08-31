use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{Asset, AssetBasic, AssetFull, AssetId, WalletId};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemAssetStore: Send + Sync {
    async fn get_asset_ids(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError>;
    fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemServiceError>;
    async fn save_assets(&self, assets: Vec<AssetBasic>) -> Result<(), GemServiceError>;
    async fn save_asset(&self, asset: AssetFull) -> Result<(), GemServiceError>;
    async fn add_missing_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError>;
    async fn add_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError>;
    async fn set_buyable_assets(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError>;
    async fn set_sellable_assets(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError>;
    async fn set_swappable_assets(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError>;
    async fn set_stakeable_assets(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError>;
}
