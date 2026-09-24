use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};

use primitives::{InAppNotification, WalletId};

use super::{GemNotificationService, GemNotificationStore};
use crate::alien::AlienProvider;
use crate::api::GemDeviceApiClient;
use crate::services::device::GemDeviceKeyService;
use crate::services::error::GemServiceError;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_preferences::{GemWalletPreferencesService, testkit::MemoryWalletPreferencesStore};
use crate::services::wallet_session::{GemWalletSessionService, testkit::MemoryWalletSessionStore};
use crate::testkit::EmptyPreferences;

impl GemNotificationService {
    pub fn mock(provider: Arc<dyn AlienProvider>, store: Arc<dyn GemNotificationStore>) -> Self {
        let api = Arc::new(GemDeviceApiClient::new(provider, Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
        let preferences = Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default())));
        let session = Arc::new(GemWalletSessionService::new(Arc::new(MemoryWalletSessionStore::default()), Arc::new(MemoryWalletStore::default())));
        session.set_current_wallet_id(Some(WalletId::Multicoin("wallet".to_string()))).unwrap();
        Self::new(api, store, preferences, session)
    }
}

#[derive(Default)]
pub struct MemoryNotificationStore {
    pub unread: bool,
    pub saved: Mutex<Vec<InAppNotification>>,
    pub read_before: Mutex<Vec<DateTime<Utc>>>,
}

#[async_trait::async_trait]
impl GemNotificationStore for MemoryNotificationStore {
    async fn save_notifications(&self, notifications: Vec<InAppNotification>) -> Result<(), GemServiceError> {
        self.saved.lock().unwrap().extend(notifications);
        Ok(())
    }

    async fn mark_notifications_read(&self, _wallet_id: WalletId, created_before: DateTime<Utc>) -> Result<(), GemServiceError> {
        self.read_before.lock().unwrap().push(created_before);
        Ok(())
    }

    async fn has_unread_notifications(&self, _wallet_id: WalletId) -> Result<bool, GemServiceError> {
        Ok(self.unread)
    }
}
