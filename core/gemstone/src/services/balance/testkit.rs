use std::sync::Mutex;

use num_bigint::BigUint;
use primitives::{AssetId, Chain, WalletId};

use super::GemAssetBalance;
use super::model::GemBalanceRecord;
use super::store::GemBalanceStore;
use crate::services::error::GemServiceError;

impl GemAssetBalance {
    pub fn mock() -> Self {
        GemAssetBalance {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            available: BigUint::ZERO,
            frozen: BigUint::ZERO,
            locked: BigUint::ZERO,
            staked: BigUint::ZERO,
            pending: BigUint::ZERO,
            pending_unconfirmed: BigUint::ZERO,
            rewards: BigUint::ZERO,
            reserved: BigUint::ZERO,
            withdrawable: BigUint::ZERO,
            earn: BigUint::ZERO,
            metadata: None,
            is_active: true,
        }
    }
}

#[derive(Default)]
pub struct RecordingBalanceStore {
    pub enabled_asset_ids: Mutex<Vec<AssetId>>,
    pub balance_writes: Mutex<Vec<Vec<GemBalanceRecord>>>,
    pub enable_writes: Mutex<Vec<(Vec<AssetId>, bool)>>,
    pub pin_writes: Mutex<Vec<(AssetId, bool)>>,
}

#[async_trait::async_trait]
impl GemBalanceStore for RecordingBalanceStore {
    async fn get_available_balances(&self, _: WalletId, _: Vec<AssetId>) -> Result<Vec<GemAssetBalance>, GemServiceError> {
        Ok(Vec::new())
    }
    async fn update_balances(&self, _: WalletId, balances: Vec<GemBalanceRecord>) -> Result<(), GemServiceError> {
        self.balance_writes.lock().unwrap().push(balances);
        Ok(())
    }
    async fn get_enabled_asset_ids(&self, _: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(self.enabled_asset_ids.lock().unwrap().clone())
    }
    async fn set_assets_enabled(&self, _: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.enable_writes.lock().unwrap().push((asset_ids, enabled));
        Ok(())
    }
    async fn set_asset_pinned(&self, _: WalletId, asset_id: AssetId, pinned: bool) -> Result<(), GemServiceError> {
        self.pin_writes.lock().unwrap().push((asset_id, pinned));
        Ok(())
    }
}
