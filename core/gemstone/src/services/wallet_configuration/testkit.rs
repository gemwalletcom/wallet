use std::sync::Arc;

use super::GemWalletConfigurationService;
use crate::alien::AlienProvider;
use crate::api::GemDeviceApiClient;
use crate::services::banner::GemBannerService;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_preferences::testkit::MemoryWalletPreferencesStore;

impl GemWalletConfigurationService {
    pub fn mock(provider: Arc<dyn AlienProvider>) -> Self {
        Self::new(
            Arc::new(GemDeviceApiClient::mock(provider)),
            Arc::new(GemBannerService::mock()),
            Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default()))),
        )
    }
}
