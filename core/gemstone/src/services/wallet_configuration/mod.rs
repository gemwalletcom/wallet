pub mod error;
pub mod rules;
pub mod store;

use primitives::WalletId;
use std::sync::Arc;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::banner::{GemBannerStore, rules as banner_rules};

pub use error::GemWalletConfigurationError;
pub use store::GemWalletConfigurationStore;

#[derive(uniffi::Object)]
pub struct GemWalletConfigurationService {
    api: Arc<GemDeviceApiClient>,
    banners: Arc<dyn GemBannerStore>,
    store: Arc<dyn GemWalletConfigurationStore>,
}

#[uniffi::export]
impl GemWalletConfigurationService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, banners: Arc<dyn GemBannerStore>, store: Arc<dyn GemWalletConfigurationStore>) -> Self {
        Self { api, banners, store }
    }

    pub async fn sync(&self, wallet_id: WalletId) -> Result<(), GemWalletConfigurationError> {
        if self.store.is_completed(wallet_id.clone()).await? {
            return Ok(());
        }
        let result = self.api.client.get_wallet_configuration(wallet_id.id()).await.map_err(GemApiError::from)?;
        for key in rules::multi_signature_banners(&wallet_id, &result.configuration) {
            let state = banner_rules::default_state(key.event);
            self.banners.set_state(key, state).await?;
        }
        self.store.set_completed(wallet_id).await
    }
}
