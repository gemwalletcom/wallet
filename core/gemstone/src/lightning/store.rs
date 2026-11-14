use std::sync::{Arc, Mutex, PoisonError};

use gem_spark::{LightningError, LightningSend, LightningStore};
use primitives::WalletId;

use crate::services::error::GemServiceError;
use crate::services::wallet_preferences::GemWalletPreferencesService;

static UPDATES: Mutex<()> = Mutex::new(());

pub(crate) struct WalletLightningStore {
    preferences: Arc<GemWalletPreferencesService>,
    wallet_id: WalletId,
}

impl WalletLightningStore {
    pub(crate) fn new(preferences: Arc<GemWalletPreferencesService>, wallet_id: WalletId) -> Self {
        Self { preferences, wallet_id }
    }

    fn update(&self, change: impl FnOnce(Vec<LightningSend>) -> Vec<LightningSend>) -> Result<(), LightningError> {
        let _update = UPDATES.lock().unwrap_or_else(PoisonError::into_inner);
        let sends = self.preferences.get_lightning_sends(self.wallet_id.clone()).map_err(storage_error)?;
        self.preferences.set_lightning_sends(self.wallet_id.clone(), &change(sends)).map_err(storage_error)
    }
}

fn storage_error(error: GemServiceError) -> LightningError {
    LightningError::Wallet(error.to_string())
}

impl LightningStore for WalletLightningStore {
    fn get_sends(&self) -> Result<Vec<LightningSend>, LightningError> {
        self.preferences.get_lightning_sends(self.wallet_id.clone()).map_err(storage_error)
    }

    fn add_send(&self, send: LightningSend) -> Result<(), LightningError> {
        self.update(|sends| sends.into_iter().chain([send]).collect())
    }

    fn delete_send(&self, transfer_id: &str) -> Result<(), LightningError> {
        self.update(|sends| sends.into_iter().filter(|send| send.transfer_id != transfer_id).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::wallet_preferences::testkit::MemoryWalletPreferencesStore;

    #[test]
    fn test_sends() {
        let preferences = Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default())));
        let store = WalletLightningStore::new(preferences.clone(), WalletId::mock());

        store.add_send(LightningSend::mock("1")).unwrap();
        store.add_send(LightningSend::mock("2")).unwrap();
        store.delete_send("1").unwrap();

        assert_eq!(store.get_sends().unwrap(), vec![LightningSend::mock("2")]);
        assert_eq!(WalletLightningStore::new(preferences, WalletId::Multicoin("0x2".to_string())).get_sends().unwrap(), vec![]);
    }
}
