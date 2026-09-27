pub mod rules;

use crate::services::error::GemServiceError;
use primitives::Wallet;
use std::sync::Arc;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::banner::{GemBannerService, rules as banner_rules};

use crate::services::wallet_preferences::GemWalletPreferencesService;

#[derive(uniffi::Object)]
pub struct GemWalletConfigurationService {
    api: Arc<GemDeviceApiClient>,
    banners: Arc<GemBannerService>,
    preferences: Arc<GemWalletPreferencesService>,
}

#[uniffi::export]
impl GemWalletConfigurationService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, banners: Arc<GemBannerService>, preferences: Arc<GemWalletPreferencesService>) -> Self {
        Self { api, banners, preferences }
    }
}

impl GemWalletConfigurationService {
    pub async fn sync(&self, wallet: &Wallet) -> Result<(), GemServiceError> {
        let wallet_id = wallet.id.clone();
        if !wallet.wallet_type.can_sign() || self.preferences.is_wallet_configuration_completed(wallet_id.clone())? {
            return Ok(());
        }
        let result = self.api.client.get_wallet_configuration(wallet_id.id()).await.map_err(GemApiError::from)?;
        for key in rules::externally_controlled_banners(&wallet_id, &result.configuration) {
            let state = banner_rules::default_state(key.event);
            self.banners.set_banner_state(key, state).await?;
        }
        self.preferences.set_wallet_configuration_completed(wallet_id)
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::{Platform, WalletType};

    use super::*;
    use crate::services::banner::testkit::MemoryBannerStore;
    use crate::services::device::GemDeviceKeyService;
    use crate::services::wallet_preferences::testkit::MemoryWalletPreferencesStore;
    use crate::testkit::{EmptyPreferences, TestAlienProvider};

    #[test]
    fn test_a_watch_only_wallet_is_never_checked() {
        block_on(async {
            let provider = Arc::new(TestAlienProvider::with_json_by_path(200, &[]));
            let preferences = Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default())));
            let service = GemWalletConfigurationService::new(
                Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences))))),
                Arc::new(GemBannerService::new(Arc::new(MemoryBannerStore::default()), Platform::IOS)),
                preferences,
            );
            let wallet = Wallet {
                wallet_type: WalletType::View,
                ..Wallet::mock()
            };

            service.sync(&wallet).await.unwrap();

            assert!(provider.requested_paths().is_empty());
        })
    }
}
