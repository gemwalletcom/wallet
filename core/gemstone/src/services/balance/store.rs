use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{AssetBalance, AssetId, WalletId};

use super::model::{GemAssetConfiguration, GemBalanceRecord};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemBalanceStore: Send + Sync {
    async fn get_available_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<Vec<AssetBalance>, GemServiceError>;
    async fn get_balance_asset_ids(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError>;
    async fn update_balances(&self, wallet_id: WalletId, balances: Vec<GemBalanceRecord>) -> Result<(), GemServiceError>;
    async fn get_enabled_asset_ids(&self, wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError>;
    async fn set_asset_configuration(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, configuration: GemAssetConfiguration) -> Result<(), GemServiceError>;
}
