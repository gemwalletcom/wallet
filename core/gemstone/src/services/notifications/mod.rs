use std::sync::Arc;

use crate::services::banner::GemNotificationPermissions;
use crate::services::device::GemDeviceService;
use crate::services::error_text::GemErrorText;
use crate::services::preferences::GemPreferencesService;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemPushResult {
    Stored,
    PermissionDenied,
    NotRegistered { error: GemErrorText },
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemPushState {
    pub is_enabled: bool,
    pub result: GemPushResult,
}

#[derive(uniffi::Object)]
pub struct GemNotificationsService {
    device: Arc<GemDeviceService>,
    preferences: Arc<GemPreferencesService>,
    permissions: Arc<dyn GemNotificationPermissions>,
}

#[uniffi::export]
impl GemNotificationsService {
    #[uniffi::constructor]
    pub fn new(device: Arc<GemDeviceService>, preferences: Arc<GemPreferencesService>, permissions: Arc<dyn GemNotificationPermissions>) -> Self {
        Self { device, preferences, permissions }
    }

    pub fn is_enabled(&self) -> bool {
        self.permissions.is_available() && self.preferences.is_push_notifications_enabled()
    }

    pub async fn set_enabled(&self, enabled: bool) -> GemPushState {
        let wanted = enabled && self.permissions.is_available();
        if wanted {
            match self.permissions.request_permissions_or_open_settings().await {
                Ok(true) => {}
                Ok(false) => return self.state(GemPushResult::PermissionDenied),
                Err(error) => return self.state(GemPushResult::NotRegistered { error: error.text() }),
            }
        }
        match self.device.set_push_enabled(wanted).await {
            Ok(()) => self.state(GemPushResult::Stored),
            Err(error) => self.state(GemPushResult::NotRegistered { error: error.text() }),
        }
    }
}

impl GemNotificationsService {
    fn state(&self, result: GemPushResult) -> GemPushState {
        GemPushState { is_enabled: self.is_enabled(), result }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    use async_trait::async_trait;
    use futures::executor::block_on;
    use primitives::Wallet;

    use super::*;
    use crate::api::GemDeviceApiClient;
    use crate::services::GemSubscriptionService;
    use crate::services::device::GemDeviceKeyService;
    use crate::services::device::testkit::MemoryDevicePlatform;
    use crate::services::error::GemServiceError;
    use crate::services::preferences::testkit::MemoryPreferencesStore;
    use crate::services::wallet::testkit::MemoryWalletStore;
    use crate::services::wallet_session::GemWalletSessionService;
    use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
    use crate::testkit::{EmptyPreferences, TestAlienProvider};

    #[derive(Default)]
    struct TestPermissions {
        available: AtomicBool,
        granted: AtomicBool,
        requests: AtomicU32,
    }

    #[async_trait]
    impl GemNotificationPermissions for TestPermissions {
        fn is_available(&self) -> bool {
            self.available.load(Ordering::SeqCst)
        }

        async fn request_permissions_or_open_settings(&self) -> Result<bool, GemServiceError> {
            self.requests.fetch_add(1, Ordering::SeqCst);
            Ok(self.granted.load(Ordering::SeqCst))
        }
    }

    fn service(status: u16, permissions: Arc<TestPermissions>) -> GemNotificationsService {
        let provider = Arc::new(TestAlienProvider::with_status(status));
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
        GemNotificationsService::new(device, preferences, permissions)
    }

    #[test]
    fn test_a_declined_permission_is_not_an_error_and_leaves_the_toggle_off() {
        block_on(async {
            let permissions = Arc::new(TestPermissions::default());
            permissions.available.store(true, Ordering::SeqCst);
            let service = service(503, permissions.clone());

            let state = service.set_enabled(true).await;

            assert_eq!(state.result, GemPushResult::PermissionDenied);
            assert!(!state.is_enabled);
            assert_eq!(permissions.requests.load(Ordering::SeqCst), 1);
        })
    }

    #[test]
    fn test_a_failed_registration_names_itself_and_the_next_identical_toggle_retries() {
        block_on(async {
            let permissions = Arc::new(TestPermissions::default());
            permissions.available.store(true, Ordering::SeqCst);
            permissions.granted.store(true, Ordering::SeqCst);
            let service = service(503, permissions.clone());

            let state = service.set_enabled(true).await;

            assert!(matches!(state.result, GemPushResult::NotRegistered { .. }), "{:?}", state.result);
            assert!(state.is_enabled, "the preference is stored even though the device is not registered");

            let retried = service.set_enabled(true).await;
            assert!(matches!(retried.result, GemPushResult::NotRegistered { .. }));
            assert_eq!(permissions.requests.load(Ordering::SeqCst), 2, "an identical toggle re-attempts the unfinished registration");
        })
    }

    #[test]
    fn test_turning_it_off_without_the_os_permission_asks_for_nothing() {
        block_on(async {
            let permissions = Arc::new(TestPermissions::default());
            let service = service(503, permissions.clone());

            let state = service.set_enabled(true).await;

            assert!(matches!(state.result, GemPushResult::NotRegistered { .. }));
            assert!(!state.is_enabled);
            assert_eq!(permissions.requests.load(Ordering::SeqCst), 0, "an unavailable permission is never requested");
        })
    }
}
