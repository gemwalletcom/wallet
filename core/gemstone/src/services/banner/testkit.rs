use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use primitives::{Asset, BannerState, Chain, Wallet};

use super::{GemBannerContext, GemBannerKey, GemBannerStore, GemNotificationPermissions};
use crate::services::error::GemServiceError;

impl GemBannerContext {
    pub fn mock() -> Self {
        Self {
            wallet: Some(Wallet::mock_with_chains(&[Chain::Ethereum, Chain::HyperCore])),
            asset: Some(Asset::from_chain(Chain::Ethereum)),
            is_stakeable: true,
            has_stake_balance: false,
            has_available_balance: false,
            is_asset_activated: true,
            asset_rank_score: Some(50),
            is_wallet_empty: false,
        }
    }
}

#[derive(Default)]
pub struct MemoryBannerStore {
    pub states: Mutex<HashMap<String, BannerState>>,
    pub writes: Mutex<Vec<Vec<GemBannerKey>>>,
}

#[async_trait]
impl GemBannerStore for MemoryBannerStore {
    async fn get_state(&self, key: GemBannerKey) -> Result<Option<BannerState>, GemServiceError> {
        Ok(self.states.lock().unwrap().get(&key.identifier()).copied())
    }
    async fn set_state(&self, key: GemBannerKey, state: BannerState) -> Result<(), GemServiceError> {
        self.states.lock().unwrap().insert(key.identifier(), state);
        Ok(())
    }
    async fn add_banners(&self, keys: Vec<GemBannerKey>, state: BannerState) -> Result<(), GemServiceError> {
        let mut states = self.states.lock().unwrap();
        for key in &keys {
            states.entry(key.identifier()).or_insert(state);
        }
        self.writes.lock().unwrap().push(keys);
        Ok(())
    }
}

pub struct DeniedNotificationPermissions;

#[async_trait]
impl GemNotificationPermissions for DeniedNotificationPermissions {
    fn is_available(&self) -> bool {
        false
    }
    async fn request_permissions_or_open_settings(&self) -> Result<bool, GemServiceError> {
        Ok(false)
    }
}
