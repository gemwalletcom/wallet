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
    use crate::gateway::EmptyPreferences;
    use crate::services::device::GemDeviceKeyService;
    use crate::services::wallet::testkit::MemoryWalletStore;
    use crate::services::wallet_preferences::testkit::MemoryWalletPreferencesStore;
    use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
    use crate::testkit::TestAlienProvider;
    use testkit::MemoryNotificationStore;

    async fn open(unread: bool) -> Vec<String> {
        let provider = Arc::new(TestAlienProvider::with_json(200, "[]"));
        let api = Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
        let store = Arc::new(MemoryNotificationStore { unread, ..Default::default() });
        let preferences = Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default())));
        let session = Arc::new(GemWalletSessionService::new(
            Arc::new(MemoryWalletSessionStore::default()),
            Arc::new(MemoryWalletStore::default()),
        ));
        session.set_current_wallet_id(Some(WalletId::Multicoin("wallet".to_string()))).unwrap();
        GemNotificationService::new(api, store, preferences, session).open().await.unwrap();
        provider.requested_paths()
    }

    #[test]
    fn test_open_marks_read_only_when_unread() {
        futures::executor::block_on(async {
            assert_eq!(open(false).await, vec!["/v2/devices/notifications?from_timestamp=0"]);
            assert_eq!(open(true).await, vec!["/v2/devices/notifications?from_timestamp=0", "/v2/devices/notifications/read"]);
        });
    }
}
