use crate::services::assets::GemAssetFilter;
use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{AssetId, RecentActivityType, WalletId};

use super::model::GemSwapPair;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemSwapStore: Send + Sync {
    async fn get_swap_pairs(&self, wallet_id: WalletId) -> Result<Vec<GemSwapPair>, GemServiceError>;
    async fn get_recent_asset_ids(&self, wallet_id: WalletId, types: Vec<RecentActivityType>, filters: Vec<GemAssetFilter>, limit: u32) -> Result<Vec<AssetId>, GemServiceError>;
    async fn get_asset_ids(&self, wallet_id: WalletId, filters: Vec<GemAssetFilter>, limit: u32) -> Result<Vec<AssetId>, GemServiceError>;
}
