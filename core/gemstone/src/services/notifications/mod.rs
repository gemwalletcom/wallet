#[cfg(test)]
pub(crate) mod testkit;

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

    pub async fn ask_to_enable(&self) -> Option<GemPushState> {
        if !self.permissions.is_available() || self.is_active().await || !self.preferences.should_ask_notifications() {
            return None;
        }
        Some(self.set_enabled(true).await)
    }

    pub async fn refresh(&self) -> bool {
        let _ = self.device.synchronize_if_needed().await;
        self.is_active().await
    }

    pub async fn set_enabled(&self, enabled: bool) -> GemPushState {
        let state = self.push_state(enabled).await;
        let _ = self.preferences.set_notifications_asked();
        state
    }
}

impl GemNotificationsService {
    async fn is_active(&self) -> bool {
        self.is_enabled() && self.permissions.is_granted().await
    }

    async fn push_state(&self, enabled: bool) -> GemPushState {
        let wanted = enabled && self.permissions.is_available();
        if wanted {
            match self.permissions.request_permissions_or_open_settings().await {
                Ok(true) => {}
                Ok(false) => {
                    let _ = self.device.set_push_enabled(true).await;
                    return self.state(GemPushResult::PermissionDenied).await;
                }
                Err(error) => return self.state(GemPushResult::NotRegistered { error: error.text() }).await,
            }
        }
        match self.device.set_push_enabled(wanted).await {
            Ok(()) => self.state(GemPushResult::Stored).await,
            Err(error) => self.state(GemPushResult::NotRegistered { error: error.text() }).await,
        }
    }

    async fn state(&self, result: GemPushResult) -> GemPushState {
        GemPushState { is_enabled: self.is_active().await, result }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use futures::executor::block_on;

    use super::*;
    use crate::services::banner::testkit::MemoryNotificationPermissions;
    use crate::testkit::TestAlienProvider;

    fn service(status: u16, permissions: Arc<MemoryNotificationPermissions>) -> GemNotificationsService {
        GemNotificationsService::mock(Arc::new(TestAlienProvider::with_status(status)), permissions)
    }

    #[test]
    fn test_a_declined_permission_is_not_an_error_and_leaves_the_toggle_off() {
        block_on(async {
            let permissions = Arc::new(MemoryNotificationPermissions::default());
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
            let permissions = Arc::new(MemoryNotificationPermissions::default());
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
    fn test_asks_only_when_there_is_something_to_ask_for() {
        block_on(async {
            let permissions = Arc::new(MemoryNotificationPermissions::default());
            let service = service(503, permissions.clone());

            assert!(service.ask_to_enable().await.is_none(), "nothing to ask for without the os permission");
            assert_eq!(permissions.requests.load(Ordering::SeqCst), 0);

            permissions.available.store(true, Ordering::SeqCst);
            permissions.granted.store(true, Ordering::SeqCst);

            assert!(service.ask_to_enable().await.is_some(), "the app asks to turn them on");
            assert_eq!(permissions.requests.load(Ordering::SeqCst), 1);

            assert!(service.ask_to_enable().await.is_none(), "once on, nothing is asked again");
            assert_eq!(permissions.requests.load(Ordering::SeqCst), 1);
        })
    }

    #[test]
    fn test_a_declined_ask_is_not_repeated_within_the_wait() {
        block_on(async {
            let permissions = Arc::new(MemoryNotificationPermissions::default());
            permissions.available.store(true, Ordering::SeqCst);
            let service = service(503, permissions.clone());

            let state = service.ask_to_enable().await;

            assert_eq!(state.map(|state| state.result), Some(GemPushResult::PermissionDenied));
            assert_eq!(permissions.requests.load(Ordering::SeqCst), 1);
            assert!(!service.preferences.should_ask_notifications(), "the ask is recorded, so the 30-day wait applies");

            assert!(service.ask_to_enable().await.is_none(), "opening support or adding a wallet within the wait does not ask again");
            assert_eq!(permissions.requests.load(Ordering::SeqCst), 1);
        })
    }

    #[test]
    fn test_push_the_user_turned_off_is_never_asked_for() {
        block_on(async {
            let permissions = Arc::new(MemoryNotificationPermissions::default());
            permissions.available.store(true, Ordering::SeqCst);
            permissions.granted.store(true, Ordering::SeqCst);
            let service = service(503, permissions.clone());
            service.preferences.set_push_notifications_declined(true).unwrap();

            assert!(service.ask_to_enable().await.is_none());
            assert_eq!(permissions.requests.load(Ordering::SeqCst), 0);
            assert!(!service.preferences.is_push_notifications_enabled(), "the user's choice stays");
        })
    }

    #[test]
    fn test_turning_it_off_without_the_os_permission_asks_for_nothing() {
        block_on(async {
            let permissions = Arc::new(MemoryNotificationPermissions::default());
            let service = service(503, permissions.clone());

            let state = service.set_enabled(true).await;

            assert!(matches!(state.result, GemPushResult::NotRegistered { .. }));
            assert!(!state.is_enabled);
            assert_eq!(permissions.requests.load(Ordering::SeqCst), 0, "an unavailable permission is never requested");
        })
    }

    #[test]
    fn test_a_permission_revoked_in_system_settings_turns_the_toggle_off_and_lets_the_app_ask_again() {
        block_on(async {
            let permissions = Arc::new(MemoryNotificationPermissions::default());
            permissions.available.store(true, Ordering::SeqCst);
            permissions.granted.store(true, Ordering::SeqCst);
            let service = service(503, permissions.clone());
            service.preferences.set_push_notifications_enabled(true).unwrap();
            assert!(service.is_active().await);

            permissions.granted.store(false, Ordering::SeqCst);

            assert!(!service.is_active().await, "the toggle follows the system permission, not the stored preference");
            assert!(service.ask_to_enable().await.is_some(), "a new wallet asks again");
            assert_eq!(permissions.requests.load(Ordering::SeqCst), 1);
        })
    }

    #[test]
    fn test_a_permission_granted_after_the_user_turned_push_on_shows_it_on() {
        block_on(async {
            let permissions = Arc::new(MemoryNotificationPermissions::default());
            permissions.available.store(true, Ordering::SeqCst);
            let service = service(200, permissions.clone());

            assert_eq!(service.set_enabled(true).await.result, GemPushResult::PermissionDenied);
            assert!(service.is_enabled(), "the choice is stored without the permission");
            assert!(!service.refresh().await, "no permission yet");

            permissions.granted.store(true, Ordering::SeqCst);

            assert!(service.refresh().await, "the user's choice is kept while they grant it in system settings");
        })
    }

    #[test]
    fn test_a_permission_granted_after_the_user_turned_push_off_keeps_it_off() {
        block_on(async {
            let permissions = Arc::new(MemoryNotificationPermissions::default());
            permissions.available.store(true, Ordering::SeqCst);
            let service = service(200, permissions.clone());

            service.set_enabled(true).await;
            service.set_enabled(false).await;
            permissions.granted.store(true, Ordering::SeqCst);

            assert!(!service.refresh().await);
        })
    }
}
