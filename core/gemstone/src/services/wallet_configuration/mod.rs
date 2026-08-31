pub mod rules;

use crate::services::error::GemServiceError;
use primitives::WalletId;
use std::sync::Arc;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::banner::{GemBannerStore, rules as banner_rules};

use crate::services::wallet_preferences::GemWalletPreferencesService;

#[derive(uniffi::Object)]
pub struct GemWalletConfigurationService {
    api: Arc<GemDeviceApiClient>,
    banners: Arc<dyn GemBannerStore>,
    preferences: Arc<GemWalletPreferencesService>,
}

#[uniffi::export]
impl GemWalletConfigurationService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, banners: Arc<dyn GemBannerStore>, preferences: Arc<GemWalletPreferencesService>) -> Self {
        Self { api, banners, preferences }
    }

    pub async fn sync(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        if self.preferences.is_wallet_configuration_completed(wallet_id.clone())? {
            return Ok(());
        }
        let result = self.api.client.get_wallet_configuration(wallet_id.id()).await.map_err(GemApiError::from)?;
        for key in rules::multi_signature_banners(&wallet_id, &result.configuration) {
            let state = banner_rules::default_state(key.event);
            self.banners.set_state(key, state).await?;
        }
        self.preferences.set_wallet_configuration_completed(wallet_id)
    }
}
