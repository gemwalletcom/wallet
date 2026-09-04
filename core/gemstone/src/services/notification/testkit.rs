use std::sync::Mutex;

use primitives::{InAppNotification, WalletId};

use super::GemNotificationStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryNotificationStore {
    pub unread: bool,
    pub saved: Mutex<Vec<InAppNotification>>,
}

#[async_trait::async_trait]
impl GemNotificationStore for MemoryNotificationStore {
    async fn save_notifications(&self, notifications: Vec<InAppNotification>) -> Result<(), GemServiceError> {
        self.saved.lock().unwrap().extend(notifications);
        Ok(())
    }

    async fn has_unread_notifications(&self, _wallet_id: WalletId) -> Result<bool, GemServiceError> {
        Ok(self.unread)
    }
}
