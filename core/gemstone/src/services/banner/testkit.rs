use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use primitives::{Asset, BannerState, Chain, Platform, Wallet};

use super::{GemBannerContext, GemBannerKey, GemBannerService, GemBannerStore, GemNotificationPermissions};
use crate::services::error::GemServiceError;

impl GemBannerService {
    pub fn mock() -> Self {
        Self::new(Arc::new(MemoryBannerStore::default()), Platform::IOS)
    }
}

impl GemBannerContext {
    pub fn mock() -> Self {
        Self {
            wallet: Some(Wallet::mock_with_chains(&[Chain::Ethereum, Chain::HyperCore])),
            asset: Some(Asset::from_chain(Chain::Ethereum)),
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
    pub state_writes: Mutex<usize>,
}

#[async_trait]
impl GemBannerStore for MemoryBannerStore {
    async fn get_state(&self, key: GemBannerKey) -> Result<Option<BannerState>, GemServiceError> {
        Ok(self.states.lock().unwrap().get(&key.identifier()).copied())
    }
    async fn set_state(&self, key: GemBannerKey, state: BannerState) -> Result<(), GemServiceError> {
        self.states.lock().unwrap().insert(key.identifier(), state);
        *self.state_writes.lock().unwrap() += 1;
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

#[derive(Default)]
pub struct MemoryNotificationPermissions {
    pub available: AtomicBool,
    pub granted: AtomicBool,
    pub requests: AtomicU32,
}

impl MemoryNotificationPermissions {
    pub fn granted() -> Self {
        Self {
            available: AtomicBool::new(true),
            granted: AtomicBool::new(true),
            ..Self::default()
        }
    }
}

#[async_trait]
impl GemNotificationPermissions for MemoryNotificationPermissions {
    fn is_available(&self) -> bool {
        self.available.load(Ordering::SeqCst)
    }
    async fn is_granted(&self) -> bool {
        self.granted.load(Ordering::SeqCst)
    }
    async fn request_permissions_or_open_settings(&self) -> Result<bool, GemServiceError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        Ok(self.granted.load(Ordering::SeqCst))
    }
}
