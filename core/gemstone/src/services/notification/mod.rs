pub mod model;
pub use model::{GemNotificationIcon, GemNotificationRow, notification_row};

pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use primitives::WalletId;
use primitives::unix_seconds;
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

    pub async fn open(&self) -> Result<(), GemServiceError> {
        let wallet_id = self.session.current_wallet_id()?;
        self.sync(wallet_id.clone()).await?;
        if self.store.has_unread_notifications(wallet_id).await? {
            self.api.client.mark_notifications_read().await.map_err(GemApiError::from)?;
        }
        Ok(())
    }
}

impl GemNotificationService {
    async fn sync(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let started_at = unix_seconds().map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
        let from_timestamp = self.preferences.get_notifications_timestamp(wallet_id.clone());
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
}
