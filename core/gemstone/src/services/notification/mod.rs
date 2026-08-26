use std::sync::Arc;

use primitives::InAppNotification;

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemNotificationService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemNotificationService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_notifications(&self, from_timestamp: u64) -> Result<Vec<InAppNotification>, GemApiError> {
        Ok(self.api.client.get_notifications(from_timestamp).await?)
    }

    pub async fn mark_read(&self) -> Result<(), GemApiError> {
        Ok(self.api.client.mark_notifications_read().await?)
    }
}
