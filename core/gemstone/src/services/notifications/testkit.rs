use std::sync::Arc;

use primitives::Wallet;

use super::GemNotificationsService;
use crate::api::GemDeviceApiClient;
use crate::services::banner::GemNotificationPermissions;
use crate::services::device::GemDeviceService;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::TestAlienProvider;

impl GemNotificationsService {
    pub fn mock(provider: Arc<TestAlienProvider>, permissions: Arc<dyn GemNotificationPermissions>) -> Self {
        let preferences = Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default())));
        let device_api = Arc::new(GemDeviceApiClient::mock(provider));
        let wallets = Arc::new(MemoryWalletStore::with_wallets(vec![Wallet::mock()]));
        let session = Arc::new(GemWalletSessionService::new(Arc::new(MemoryWalletSessionStore::default()), wallets));
        let device = GemDeviceService::mock(device_api, session, preferences.clone());
        Self::new(device, preferences, permissions)
    }
}
