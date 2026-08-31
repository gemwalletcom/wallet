use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{NFTAssetData, NFTAssetId, NFTData, WalletId};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemNftStore: Send + Sync {
    async fn save_nfts(&self, wallet_id: WalletId, data: Vec<NFTData>) -> Result<(), GemServiceError>;
    async fn get_asset_data(&self, asset_id: NFTAssetId) -> Result<Option<NFTAssetData>, GemServiceError>;
    async fn save_asset(&self, data: NFTAssetData) -> Result<(), GemServiceError>;
}
