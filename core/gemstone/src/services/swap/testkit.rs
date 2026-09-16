use std::sync::Mutex;

use primitives::{AssetId, Chain, WalletId};

use super::model::GemSwapPair;
use super::store::GemSwapStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemorySwapStore {
    pub pairs: Mutex<Vec<GemSwapPair>>,
    pub recent_asset_ids: Mutex<Vec<AssetId>>,
    pub pay_asset_ids: Mutex<Vec<AssetId>>,
    pub receive_asset_ids: Mutex<Vec<AssetId>>,
    pub receive_requests: Mutex<Vec<(Vec<Chain>, Vec<AssetId>)>>,
}

#[async_trait::async_trait]
impl GemSwapStore for MemorySwapStore {
    async fn get_swap_pairs(&self, _wallet_id: WalletId) -> Result<Vec<GemSwapPair>, GemServiceError> {
        Ok(self.pairs.lock().unwrap().clone())
    }

    async fn get_recent_asset_ids(&self, _wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(self.recent_asset_ids.lock().unwrap().clone())
    }

    async fn get_pay_asset_ids(&self, _wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(self.pay_asset_ids.lock().unwrap().clone())
    }

    async fn get_receive_asset_ids(&self, _wallet_id: WalletId, chains: Vec<Chain>, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        self.receive_requests.lock().unwrap().push((chains, asset_ids));
        Ok(self.receive_asset_ids.lock().unwrap().clone())
    }
}
