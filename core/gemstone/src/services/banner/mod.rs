pub mod model;
pub mod permissions;
pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{Asset, BannerEvent, BannerState, Wallet};

pub use model::{GemBannerButton, GemBannerContent, GemBannerContext, GemBannerDescription, GemBannerDestination, GemBannerIcon, GemBannerItem, GemBannerKey, GemBannerLink, GemBannerRow, GemBannerStyle, GemBannerTitle};
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
}

impl GemBannerService {
    pub async fn setup_wallet(&self, wallet: Wallet) -> Result<(), GemServiceError> {
        self.add_missing_banners(rules::wallet_setup_keys(&wallet)).await
    }

    pub async fn close(&self, key: GemBannerKey) -> Result<(), GemServiceError> {
        self.set_banner_state(key, BannerState::Cancelled).await
    }

    pub async fn set_banner_state(&self, key: GemBannerKey, state: BannerState) -> Result<(), GemServiceError> {
        self.store.set_state(key, state).await
    }

    pub fn banner_content(&self, event: BannerEvent, asset: Option<Asset>, state: BannerState) -> GemBannerContent {
        rules::banner_content(event, asset.as_ref(), state)
    }

    pub async fn setup(&self) -> Result<(), GemServiceError> {
        self.add_missing_banners(rules::setup_keys()).await
    }

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
    use super::testkit::MemoryBannerStore;
    use super::*;
    use futures::executor::block_on;
    use primitives::{Chain, WalletSource};

    #[test]
    fn test_wallet_setup_writes_its_banners_once() {
        block_on(async {
            let store = Arc::new(MemoryBannerStore::default());
            let service = GemBannerService::new(store.clone());
            let wallet = Wallet {
                source: WalletSource::Create,
                ..Wallet::mock_with_chains(&[Chain::Xrp, Chain::Ethereum])
            };

            service.setup_wallet(wallet.clone()).await.unwrap();
            service.setup_wallet(wallet.clone()).await.unwrap();

            let writes = store.writes.lock().unwrap();
            assert_eq!(writes.len(), 1);
            assert_eq!(writes[0].len(), rules::wallet_setup_keys(&wallet).len());
            assert!(writes[0].iter().any(|key| key.event == BannerEvent::Onboarding));
        });
    }

    #[test]
    fn test_a_dismissed_banner_is_not_recreated_by_setup() {
        block_on(async {
            let store = Arc::new(MemoryBannerStore::default());
            let service = GemBannerService::new(store.clone());
            let wallet = Wallet {
                source: WalletSource::Import,
                ..Wallet::mock_with_chains(&[Chain::Xrp, Chain::Ethereum])
            };
            service.setup_wallet(wallet.clone()).await.unwrap();
            let key = rules::wallet_setup_keys(&wallet).remove(0);
            service.close(key.clone()).await.unwrap();

            service.setup_wallet(wallet).await.unwrap();

            assert_eq!(store.get_state(key).await.unwrap(), Some(BannerState::Cancelled));
            assert_eq!(store.writes.lock().unwrap().len(), 1);
        });
    }
}
