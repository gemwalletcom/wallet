pub mod model;
pub use model::{GemNotificationIcon, GemNotificationRow, notification_row};

pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::models::state::GemLoadState;
use crate::services::error::GemServiceError;
use chrono::DateTime;
use primitives::unix_seconds;
use primitives::{InAppNotification, WalletId};
use std::sync::Arc;

use crate::api::{GemApiError, GemDeviceApiClient};

pub use store::GemNotificationStore;

use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemNotificationService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemNotificationStore>,
    preferences: Arc<GemWalletPreferencesService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemNotificationService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, store: Arc<dyn GemNotificationStore>, preferences: Arc<GemWalletPreferencesService>, session: Arc<GemWalletSessionService>) -> Self {
        Self { api, store, preferences, session }
    }

    pub async fn refresh(&self, has_notifications: bool) -> GemLoadState {
        GemLoadState::refreshed(self.open().await, has_notifications)
    }
}

impl GemNotificationService {
    pub async fn save_notifications(&self, notifications: Vec<InAppNotification>) -> Result<(), GemServiceError> {
        self.store.save_notifications(notifications).await
    }

    async fn open(&self) -> Result<(), GemServiceError> {
        let wallet_id = self.session.current_wallet_id()?;
        let last_visit = self.preferences.get_notifications_timestamp(wallet_id.clone());
        self.store.mark_notifications_read(wallet_id.clone(), DateTime::from_timestamp(last_visit as i64, 0).unwrap_or_default()).await?;
        self.sync(wallet_id.clone(), last_visit).await?;
        if self.store.has_unread_notifications(wallet_id).await? {
            self.api.client.mark_notifications_read().await.map_err(GemApiError::from)?;
        }
        Ok(())
    }

    async fn sync(&self, wallet_id: WalletId, from_timestamp: u64) -> Result<(), GemServiceError> {
        let started_at = unix_seconds().map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
        let notifications = self.api.client.get_notifications(from_timestamp).await.map_err(GemApiError::from)?;
        self.store.save_notifications(notifications).await?;
        self.preferences.set_notifications_timestamp(wallet_id, started_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::TestAlienProvider;
    use testkit::MemoryNotificationStore;

    #[test]
    fn test_open_marks_read_only_when_unread() {
        futures::executor::block_on(async {
            let read = Arc::new(TestAlienProvider::with_json(200, "[]"));
            GemNotificationService::mock(read.clone(), Arc::new(MemoryNotificationStore::default())).open().await.unwrap();
            assert_eq!(read.requested_paths(), vec!["/v2/devices/notifications?from_timestamp=0"]);

            let unread = Arc::new(TestAlienProvider::with_json(200, "[]"));
            GemNotificationService::mock(unread.clone(), Arc::new(MemoryNotificationStore { unread: true, ..Default::default() })).open().await.unwrap();
            assert_eq!(unread.requested_paths(), vec!["/v2/devices/notifications?from_timestamp=0", "/v2/devices/notifications/read"]);
        });
    }

    #[test]
    fn test_open_clears_what_arrived_before_the_last_visit() {
        futures::executor::block_on(async {
            let store = Arc::new(MemoryNotificationStore::default());
            let service = GemNotificationService::mock(Arc::new(TestAlienProvider::with_json(200, "[]")), store.clone());
            service.open().await.unwrap();
            let last_visit = service.preferences.get_notifications_timestamp(service.session.current_wallet_id().unwrap());
            service.open().await.unwrap();

            let read_before = store.read_before.lock().unwrap().iter().map(|date| date.timestamp() as u64).collect::<Vec<_>>();
            assert_eq!(read_before, vec![0, last_visit]);
        });
    }
}
