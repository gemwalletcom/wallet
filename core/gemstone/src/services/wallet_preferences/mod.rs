pub mod model;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use primitives::{AssetId, PerpetualAccountMode, WalletId};

use strum::AsRefStr;

use crate::services::error::GemServiceError;
pub use model::GemDiscoveryStep;
pub use store::GemWalletPreferencesStore;

#[derive(Debug, Clone, Copy, AsRefStr)]
#[strum(serialize_all = "snake_case")]
enum WalletPreferenceKey {
    AssetsTimestamp,
    #[strum(serialize = "transactions_timestamp_v1")]
    TransactionsTimestamp,
    #[strum(serialize = "transactions_for_asset_v1")]
    TransactionsForAssetTimestamp,
    NotificationsTimestamp,
    CompleteInitialLoadAssets,
    CompleteInitialLoadTransactions,
    CompleteInitialLoadNfts,
    CompleteInitialWalletConfiguration,
    PerpetualAccountMode,
}

#[derive(uniffi::Object)]
pub struct GemWalletPreferencesService {
    store: Arc<dyn GemWalletPreferencesStore>,
}

#[uniffi::export]
impl GemWalletPreferencesService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemWalletPreferencesStore>) -> Self {
        Self { store }
    }

    pub fn includes_perpetual_collateral(&self, wallet_id: WalletId) -> bool {
        let mode = self.get_perpetual_account_mode(wallet_id).unwrap_or(PerpetualAccountMode::Standard);
        crate::services::perpetual::rules::includes_perpetual_collateral(mode)
    }

    pub fn get_perpetual_account_mode(&self, wallet_id: WalletId) -> Result<PerpetualAccountMode, GemServiceError> {
        Ok(match self.store.get(wallet_id, WalletPreferenceKey::PerpetualAccountMode.as_ref().to_string()).as_deref() {
            Some("unified") => PerpetualAccountMode::Unified,
            _ => PerpetualAccountMode::Standard,
        })
    }
}

impl GemWalletPreferencesService {
    pub fn get_assets_timestamp(&self, wallet_id: WalletId) -> u64 {
        self.get_timestamp(wallet_id, WalletPreferenceKey::AssetsTimestamp)
    }

    pub fn is_initial_load_completed(&self, wallet_id: WalletId, step: GemDiscoveryStep) -> Result<bool, GemServiceError> {
        self.get_flag(wallet_id, initial_load_key(step))
    }
}

impl GemWalletPreferencesService {
    pub fn reset_transactions_timestamp(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.store
            .set(wallet_id.clone(), WalletPreferenceKey::TransactionsTimestamp.as_ref().to_string(), 0.to_string())?;
        self.store
            .set(wallet_id, WalletPreferenceKey::CompleteInitialLoadTransactions.as_ref().to_string(), "false".to_string())
    }

    pub fn delete_preferences(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.store.delete_preferences(wallet_id)
    }

    pub fn set_assets_timestamp(&self, wallet_id: WalletId, timestamp: u64) -> Result<(), GemServiceError> {
        self.set_value(wallet_id, WalletPreferenceKey::AssetsTimestamp.as_ref().to_string(), timestamp.to_string())
    }

    pub fn get_transactions_timestamp(&self, wallet_id: WalletId, asset_id: Option<AssetId>) -> u64 {
        self.get_timestamp_key(wallet_id, transactions_key(asset_id))
    }

    pub fn set_transactions_timestamp(&self, wallet_id: WalletId, asset_id: Option<AssetId>, timestamp: u64) -> Result<(), GemServiceError> {
        self.set_value(wallet_id, transactions_key(asset_id), timestamp.to_string())
    }

    pub fn get_notifications_timestamp(&self, wallet_id: WalletId) -> u64 {
        self.get_timestamp(wallet_id, WalletPreferenceKey::NotificationsTimestamp)
    }

    pub fn set_notifications_timestamp(&self, wallet_id: WalletId, timestamp: u64) -> Result<(), GemServiceError> {
        self.store
            .set(wallet_id, WalletPreferenceKey::NotificationsTimestamp.as_ref().to_string(), timestamp.to_string())
    }

    pub fn set_initial_load_completed(&self, wallet_id: WalletId, step: GemDiscoveryStep) -> Result<(), GemServiceError> {
        self.set_flag(wallet_id, initial_load_key(step))
    }

    pub fn is_wallet_configuration_completed(&self, wallet_id: WalletId) -> Result<bool, GemServiceError> {
        self.get_flag(wallet_id, WalletPreferenceKey::CompleteInitialWalletConfiguration)
    }

    pub fn set_wallet_configuration_completed(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.set_flag(wallet_id, WalletPreferenceKey::CompleteInitialWalletConfiguration)
    }

    pub fn complete_initial_synchronization(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.set_flag(wallet_id.clone(), WalletPreferenceKey::CompleteInitialWalletConfiguration)?;
        for step in [GemDiscoveryStep::Assets, GemDiscoveryStep::Transactions, GemDiscoveryStep::Nfts] {
            self.set_flag(wallet_id.clone(), initial_load_key(step))?;
        }
        Ok(())
    }

    pub fn set_perpetual_account_mode(&self, wallet_id: WalletId, mode: PerpetualAccountMode) -> Result<(), GemServiceError> {
        let value = match mode {
            PerpetualAccountMode::Standard => "standard",
            PerpetualAccountMode::Unified => "unified",
        };
        self.set_value(wallet_id, WalletPreferenceKey::PerpetualAccountMode.as_ref().to_string(), value.to_string())
    }

    fn get_timestamp(&self, wallet_id: WalletId, key: WalletPreferenceKey) -> u64 {
        self.get_timestamp_key(wallet_id, key.as_ref().to_string())
    }

    fn get_timestamp_key(&self, wallet_id: WalletId, key: String) -> u64 {
        crate::services::clock::parse_timestamp_or_zero(self.store.get(wallet_id, key))
    }

    fn get_flag(&self, wallet_id: WalletId, key: WalletPreferenceKey) -> Result<bool, GemServiceError> {
        Ok(self.store.get(wallet_id, key.as_ref().to_string()).as_deref() == Some("true"))
    }

    fn set_flag(&self, wallet_id: WalletId, key: WalletPreferenceKey) -> Result<(), GemServiceError> {
        self.set_value(wallet_id, key.as_ref().to_string(), "true".to_string())
    }

    fn set_value(&self, wallet_id: WalletId, key: String, value: String) -> Result<(), GemServiceError> {
        if self.store.get(wallet_id.clone(), key.clone()).as_deref() == Some(value.as_str()) {
            return Ok(());
        }
        self.store.set(wallet_id, key, value)
    }
}

fn transactions_key(asset_id: Option<AssetId>) -> String {
    match asset_id {
        Some(asset_id) => format!("{}_{asset_id}", WalletPreferenceKey::TransactionsForAssetTimestamp.as_ref()),
        None => WalletPreferenceKey::TransactionsTimestamp.as_ref().to_string(),
    }
}

fn initial_load_key(step: GemDiscoveryStep) -> WalletPreferenceKey {
    match step {
        GemDiscoveryStep::Assets => WalletPreferenceKey::CompleteInitialLoadAssets,
        GemDiscoveryStep::Transactions => WalletPreferenceKey::CompleteInitialLoadTransactions,
        GemDiscoveryStep::Nfts => WalletPreferenceKey::CompleteInitialLoadNfts,
    }
}

#[cfg(test)]
mod tests {
    use super::testkit::MemoryWalletPreferencesStore;
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_wallet_preferences_keys_and_clear() {
        let store = Arc::new(MemoryWalletPreferencesStore::default());
        let service = GemWalletPreferencesService::new(store.clone());
        let wallet = WalletId::Multicoin("0x1".into());
        let other = WalletId::Multicoin("0x2".into());
        let asset_id = AssetId::from_chain(Chain::Ethereum);

        assert_eq!(service.get_transactions_timestamp(wallet.clone(), None), 0);
        service.set_transactions_timestamp(wallet.clone(), Some(asset_id.clone()), 42).unwrap();
        service.set_transactions_timestamp(wallet.clone(), None, 7).unwrap();
        let writes = *store.writes.lock().unwrap();
        service.set_transactions_timestamp(wallet.clone(), None, 7).unwrap();
        assert_eq!(*store.writes.lock().unwrap(), writes);
        assert_eq!(service.get_transactions_timestamp(wallet.clone(), Some(asset_id)), 42);
        assert_eq!(service.get_transactions_timestamp(wallet.clone(), None), 7);

        assert!(!service.is_initial_load_completed(wallet.clone(), GemDiscoveryStep::Nfts).unwrap());
        service.complete_initial_synchronization(wallet.clone()).unwrap();
        assert!(service.is_initial_load_completed(wallet.clone(), GemDiscoveryStep::Nfts).unwrap());
        assert!(service.is_wallet_configuration_completed(wallet.clone()).unwrap());
        assert!(!service.is_wallet_configuration_completed(other.clone()).unwrap());

        service.set_perpetual_account_mode(wallet.clone(), PerpetualAccountMode::Unified).unwrap();
        assert_eq!(service.get_perpetual_account_mode(wallet.clone()).unwrap(), PerpetualAccountMode::Unified);
        assert_eq!(service.get_perpetual_account_mode(other).unwrap(), PerpetualAccountMode::Standard);

        service.delete_preferences(wallet.clone()).unwrap();
        assert_eq!(service.get_transactions_timestamp(wallet.clone(), None), 0);
        assert!(!service.is_wallet_configuration_completed(wallet).unwrap());
    }
}
