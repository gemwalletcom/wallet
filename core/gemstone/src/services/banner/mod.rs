pub mod model;
pub mod permissions;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{Asset, BannerEvent, BannerState, Wallet};

pub use model::{
    GemBannerAction, GemBannerAmount, GemBannerContent, GemBannerContext, GemBannerDescription, GemBannerIcon, GemBannerItem, GemBannerKey, GemBannerLink, GemBannerTitle,
};
pub use permissions::GemNotificationPermissions;
pub use store::GemBannerStore;

#[derive(uniffi::Object)]
pub struct GemBannerService {
    store: Arc<dyn GemBannerStore>,
}

#[uniffi::export]
impl GemBannerService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemBannerStore>) -> Self {
        Self { store }
    }

    pub async fn setup(&self) -> Result<(), GemServiceError> {
        self.add_missing_banners(rules::setup_keys()).await
    }

    pub async fn setup_wallet(&self, wallet: Wallet) -> Result<(), GemServiceError> {
        self.add_missing_banners(rules::wallet_setup_keys(&wallet)).await
    }

    pub async fn apply_action(&self, key: GemBannerKey, action: GemBannerAction) -> Result<(), GemServiceError> {
        match action.is_dismissal() {
            true => self.close(key).await,
            false => Ok(()),
        }
    }

    pub async fn close(&self, key: GemBannerKey) -> Result<(), GemServiceError> {
        self.store.set_state(key, BannerState::Cancelled).await
    }

    pub fn banner_content(&self, event: BannerEvent, asset: Option<Asset>) -> GemBannerContent {
        rules::banner_content(event, asset.as_ref())
    }
}

impl GemBannerService {
    async fn add_missing_banners(&self, keys: Vec<GemBannerKey>) -> Result<(), GemServiceError> {
        let mut missing = Vec::new();
        for key in keys {
            if self.store.get_state(key.clone()).await?.is_none() {
                missing.push(key);
            }
        }
        if missing.is_empty() {
            return Ok(());
        }
        self.store.add_banners(missing, BannerState::Active).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use async_trait::async_trait;
    use primitives::{Account, WalletSource};
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryBannerStore {
        states: Mutex<HashMap<String, BannerState>>,
        writes: Mutex<Vec<Vec<GemBannerKey>>>,
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

    fn wallet(source: WalletSource) -> Wallet {
        Wallet {
            source,
            ..Wallet::mock_with_accounts(Account::mock_chains(&[primitives::Chain::Xrp, primitives::Chain::Ethereum], "address"))
        }
    }

    #[test]
    fn test_wallet_setup_writes_its_banners_once() {
        block_on(async {
            let store = Arc::new(MemoryBannerStore::default());
            let service = GemBannerService::new(store.clone());

            service.setup_wallet(wallet(WalletSource::Create)).await.unwrap();
            service.setup_wallet(wallet(WalletSource::Create)).await.unwrap();

            let writes = store.writes.lock().unwrap();
            assert_eq!(writes.len(), 1);
            assert_eq!(writes[0].len(), rules::wallet_setup_keys(&wallet(WalletSource::Create)).len());
            assert!(writes[0].iter().any(|key| key.event == BannerEvent::Onboarding));
        });
    }

    #[test]
    fn test_a_dismissed_banner_is_not_recreated_by_setup() {
        block_on(async {
            let store = Arc::new(MemoryBannerStore::default());
            let service = GemBannerService::new(store.clone());
            service.setup_wallet(wallet(WalletSource::Import)).await.unwrap();
            let key = rules::wallet_setup_keys(&wallet(WalletSource::Import)).remove(0);
            service.close(key.clone()).await.unwrap();

            service.setup_wallet(wallet(WalletSource::Import)).await.unwrap();

            assert_eq!(store.get_state(key).await.unwrap(), Some(BannerState::Cancelled));
            assert_eq!(store.writes.lock().unwrap().len(), 1);
        });
    }
}
