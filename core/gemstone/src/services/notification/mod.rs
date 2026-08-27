pub mod store;

use crate::services::error::GemServiceError;
use primitives::WalletId;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::api::GemDeviceApiClient;

pub use store::GemNotificationStore;

use crate::services::wallet_preferences::GemWalletPreferencesService;

#[derive(uniffi::Object)]
pub struct GemNotificationService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemNotificationStore>,
    preferences: Arc<GemWalletPreferencesService>,
}

#[uniffi::export]
impl GemNotificationService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, store: Arc<dyn GemNotificationStore>, preferences: Arc<GemWalletPreferencesService>) -> Self {
        Self { api, store, preferences }
    }

    pub async fn sync(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| GemServiceError::Store { msg: error.to_string() })?
            .as_secs();
        let from_timestamp = self.preferences.get_notifications_timestamp(wallet_id.clone())?;
        let notifications = self.api.client.get_notifications(from_timestamp).await.map_err(crate::api::GemApiError::from)?;
        self.store.save(notifications).await?;
        self.preferences.set_notifications_timestamp(wallet_id, started_at)
    }

    pub async fn mark_read(&self) -> Result<(), GemServiceError> {
        Ok(self.api.client.mark_notifications_read().await.map_err(crate::api::GemApiError::from)?)
    }
}
