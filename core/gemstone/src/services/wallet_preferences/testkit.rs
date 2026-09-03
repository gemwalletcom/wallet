use std::collections::HashMap;
use std::sync::Mutex;

use primitives::WalletId;

use super::GemWalletPreferencesStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryWalletPreferencesStore {
    pub values: Mutex<HashMap<(String, String), String>>,
    pub writes: Mutex<u32>,
}

impl GemWalletPreferencesStore for MemoryWalletPreferencesStore {
    fn get(&self, wallet_id: WalletId, key: String) -> Option<String> {
        self.values.lock().unwrap().get(&(wallet_id.id(), key)).cloned()
    }
    fn set(&self, wallet_id: WalletId, key: String, value: String) -> Result<(), GemServiceError> {
        *self.writes.lock().unwrap() += 1;
        self.values.lock().unwrap().insert((wallet_id.id(), key), value);
        Ok(())
    }
    fn delete_preferences(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.values.lock().unwrap().retain(|(id, _), _| *id != wallet_id.id());
        Ok(())
    }
}
