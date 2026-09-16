use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use primitives::BannerState;

use super::{GemBannerKey, GemBannerStore};
use crate::services::error::GemServiceError;

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
