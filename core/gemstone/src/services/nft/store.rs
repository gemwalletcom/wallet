use async_trait::async_trait;
use primitives::{NFTAssetData, NFTAssetId, NFTData};

use super::error::GemNftError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemNftStore: Send + Sync {
    async fn save(&self, wallet_id: String, data: Vec<NFTData>) -> Result<(), GemNftError>;
    async fn get_asset_data(&self, asset_id: NFTAssetId) -> Result<Option<NFTAssetData>, GemNftError>;
    async fn add_asset_data(&self, data: NFTAssetData) -> Result<(), GemNftError>;
}
