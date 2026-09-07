use std::sync::Mutex;

use async_trait::async_trait;
use primitives::{Asset, AssetBasic, AssetFull, AssetId, WalletId};

use super::GemAssetStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryAssetStore {
    assets: Mutex<Vec<Asset>>,
}

#[async_trait]
impl GemAssetStore for MemoryAssetStore {
    async fn get_asset_ids(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(self.get_assets(asset_ids).await?.into_iter().map(|asset| asset.id).collect())
    }
    async fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemServiceError> {
        Ok(self.assets.lock().unwrap().iter().filter(|asset| asset_ids.contains(&asset.id)).cloned().collect())
    }
    async fn save_assets(&self, assets: Vec<AssetBasic>) -> Result<(), GemServiceError> {
        self.assets.lock().unwrap().extend(assets.into_iter().map(|asset| asset.asset));
        Ok(())
    }
    async fn save_asset(&self, asset: AssetFull) -> Result<(), GemServiceError> {
        self.assets.lock().unwrap().push(asset.asset);
        Ok(())
    }
    async fn add_missing_balances(&self, _wallet_id: WalletId, _asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn add_balances(&self, _wallet_id: WalletId, _asset_ids: Vec<AssetId>, _enabled: bool) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn set_buyable_assets(&self, _asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn set_sellable_assets(&self, _asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn set_swappable_assets(&self, _asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn set_stakeable_assets(&self, _asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        Ok(())
    }
}
