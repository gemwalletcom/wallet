pub mod error;
pub mod store;

use primitives::WalletId;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::api::GemDeviceApiClient;

pub use error::GemNotificationError;
pub use store::GemNotificationStore;

#[derive(uniffi::Object)]
pub struct GemNotificationService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemNotificationStore>,
}

#[uniffi::export]
impl GemNotificationService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, store: Arc<dyn GemNotificationStore>) -> Self {
        Self { api, store }
    }

    pub async fn sync(&self, wallet_id: WalletId) -> Result<(), GemNotificationError> {
        let started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| GemNotificationError::Store { msg: error.to_string() })?
            .as_secs();
        let from_timestamp = self.store.get_sync_timestamp(wallet_id.clone()).await?;
        let notifications = self.api.client.get_notifications(from_timestamp).await.map_err(crate::api::GemApiError::from)?;
        self.store.save(notifications).await?;
        self.store.set_sync_timestamp(wallet_id, started_at).await
    }

    pub async fn mark_read(&self) -> Result<(), GemNotificationError> {
        Ok(self.api.client.mark_notifications_read().await.map_err(crate::api::GemApiError::from)?)
    }
}
