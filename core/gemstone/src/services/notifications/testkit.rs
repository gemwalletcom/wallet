use std::sync::Arc;

use primitives::Wallet;

use super::GemNotificationsService;
use crate::api::GemDeviceApiClient;
use crate::services::GemSubscriptionService;
use crate::services::banner::GemNotificationPermissions;
use crate::services::device::testkit::MemoryDevicePlatform;
use crate::services::device::{GemDeviceKeyService, GemDeviceService};
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::{EmptyPreferences, TestAlienProvider};

impl GemNotificationsService {
    pub fn mock(provider: Arc<TestAlienProvider>, permissions: Arc<dyn GemNotificationPermissions>) -> Self {
        let preferences = Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default())));
        let device_api = Arc::new(GemDeviceApiClient::new(provider, Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
        let wallets = Arc::new(MemoryWalletStore {
            wallets: std::sync::Mutex::new(vec![Wallet::mock()]),
            ..Default::default()
        });
        let session = Arc::new(GemWalletSessionService::new(Arc::new(MemoryWalletSessionStore::default()), wallets));
        let device = Arc::new(GemDeviceService::new(
            device_api.clone(),
            Arc::new(GemSubscriptionService::new(device_api, session.clone())),
            session,
            Arc::new(MemoryDevicePlatform),
            preferences.clone(),
        ));
        Self::new(device, preferences, permissions)
    }
}
