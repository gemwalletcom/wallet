use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{AssetId, WalletId};

use super::model::{GemAssetBalance, GemBalanceUpdate};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemBalanceStore: Send + Sync {
    async fn get_available_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<Vec<GemAssetBalance>, GemServiceError>;
    async fn update_balances(&self, wallet_id: WalletId, updates: Vec<GemBalanceUpdate>) -> Result<(), GemServiceError>;
    async fn get_enabled_asset_ids(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError>;
    async fn set_assets_enabled(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError>;
    async fn set_asset_pinned(&self, wallet_id: WalletId, asset_id: AssetId, pinned: bool) -> Result<(), GemServiceError>;
}
