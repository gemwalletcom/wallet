use std::sync::Mutex;

use primitives::{NFTAssetData, NFTAssetId, NFTData, WalletId};

use super::GemNftStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryNftStore {
    pub cached: Option<NFTAssetData>,
    pub added: Mutex<Vec<NFTAssetData>>,
    pub saved: Mutex<Vec<(WalletId, Vec<NFTData>)>>,
}

#[async_trait::async_trait]
impl GemNftStore for MemoryNftStore {
    async fn save_nfts(&self, wallet_id: WalletId, data: Vec<NFTData>) -> Result<(), GemServiceError> {
        self.saved.lock().unwrap().push((wallet_id, data));
        Ok(())
    }
    async fn get_asset_data(&self, _asset_id: NFTAssetId) -> Result<Option<NFTAssetData>, GemServiceError> {
        Ok(self.cached.clone())
    }
    async fn save_asset(&self, data: NFTAssetData) -> Result<(), GemServiceError> {
        self.added.lock().unwrap().push(data);
        Ok(())
    }
}
