use std::sync::Mutex;

use async_trait::async_trait;
use primitives::{Asset, AssetBasic, AssetFull, AssetId, WalletId};

use super::GemAssetStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryAssetStore {
    pub assets: Mutex<Vec<AssetBasic>>,
    pub added_balances: Mutex<Vec<(WalletId, Vec<AssetId>, bool)>>,
}

#[async_trait]
impl GemAssetStore for MemoryAssetStore {
    async fn get_asset_ids(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(self.get_assets(asset_ids).await?.into_iter().map(|asset| asset.id).collect())
    }
    async fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemServiceError> {
        Ok(self
            .assets
            .lock()
            .unwrap()
            .iter()
            .filter(|basic| asset_ids.contains(&basic.asset.id))
            .map(|basic| basic.asset.clone())
            .collect())
    }
    async fn save_assets(&self, assets: Vec<AssetBasic>) -> Result<(), GemServiceError> {
        self.assets.lock().unwrap().extend(assets);
        Ok(())
    }
    async fn save_asset(&self, asset: AssetFull) -> Result<(), GemServiceError> {
        self.assets.lock().unwrap().push(AssetBasic::new(asset.asset, asset.properties, asset.score));
        Ok(())
    }
    async fn add_missing_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.added_balances.lock().unwrap().push((wallet_id, asset_ids, false));
        Ok(())
    }
    async fn add_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.added_balances.lock().unwrap().push((wallet_id, asset_ids, enabled));
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
