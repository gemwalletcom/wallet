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
        if self.preferences.is_wallet_configuration_completed(wallet_id.clone())? {
            return Ok(());
        }
        self.refresh(wallet).await
    }

    pub async fn refresh(&self, wallet: &Wallet) -> Result<(), GemServiceError> {
        let wallet_id = wallet.id.clone();
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
    use primitives::{Platform, WalletConfiguration, WalletConfigurationResult, WalletType};

    use super::*;
    use crate::services::banner::testkit::MemoryBannerStore;
    use crate::services::device::GemDeviceKeyService;
    use crate::services::wallet_preferences::testkit::MemoryWalletPreferencesStore;
    use crate::testkit::{EmptyPreferences, TestAlienProvider};

    #[test]
    fn test_sync_checks_watch_only_wallet() {
        block_on(async {
            let wallet = Wallet {
                wallet_type: WalletType::View,
                ..Wallet::mock()
            };
            let result = WalletConfigurationResult {
                wallet_id: wallet.id.clone(),
                configuration: WalletConfiguration {
                    multi_signature_accounts: vec![],
                    externally_controlled_accounts: vec![],
                },
            };
            let provider = Arc::new(TestAlienProvider::with_json_by_path(200, &[("wallet_configuration", &serde_json::to_string(&result).unwrap())]));
            let preferences = Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default())));
            let service = GemWalletConfigurationService::new(
                Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences))))),
                Arc::new(GemBannerService::new(Arc::new(MemoryBannerStore::default()), Platform::IOS)),
                preferences,
            );

            service.sync(&wallet).await.unwrap();

            assert_eq!(provider.requested_paths(), vec!["/v2/devices/wallet_configuration"]);
        })
    }

    #[test]
    fn test_refresh_checks_any_completed_wallet_configuration() {
        block_on(async {
            let wallet = Wallet {
                wallet_type: WalletType::View,
                ..Wallet::mock()
            };
            let result = WalletConfigurationResult {
                wallet_id: wallet.id.clone(),
                configuration: WalletConfiguration {
                    multi_signature_accounts: vec![],
                    externally_controlled_accounts: vec![],
                },
            };
            let provider = Arc::new(TestAlienProvider::with_json_by_path(200, &[("wallet_configuration", &serde_json::to_string(&result).unwrap())]));
            let preferences = Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default())));
            preferences.set_wallet_configuration_completed(wallet.id.clone()).unwrap();
            let service = GemWalletConfigurationService::new(
                Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences))))),
                Arc::new(GemBannerService::new(Arc::new(MemoryBannerStore::default()), Platform::IOS)),
                preferences,
            );

            service.refresh(&wallet).await.unwrap();

            assert_eq!(provider.requested_paths(), vec!["/v2/devices/wallet_configuration"]);
        })
    }
}
