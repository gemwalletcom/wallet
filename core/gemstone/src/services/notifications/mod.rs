use std::sync::Arc;

use crate::services::banner::GemNotificationPermissions;
use crate::services::device::GemDeviceService;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;

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
        self.preferences.is_push_notifications_enabled()
    }

    pub async fn set_enabled(&self, enabled: bool) -> Result<bool, GemServiceError> {
        if !enabled {
            self.device.set_push_enabled(false).await?;
            return Ok(false);
        }
        let granted = self.permissions.request_permissions_or_open_settings().await?;
        if granted {
            self.device.synchronize_if_needed().await?;
        }
        Ok(granted)
    }
}
