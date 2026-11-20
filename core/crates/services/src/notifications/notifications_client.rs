use std::error::Error;
use std::sync::Arc;

use in_app_notifications::map_notification;
use localizer::LanguageLocalizer;
use primitives::{DateTime, InAppNotification};

use super::repository::Repository;

#[derive(Clone)]
pub struct NotificationsClient {
    repository: Arc<dyn Repository>,
}

impl NotificationsClient {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn get_notifications(&self, device_id: &str, since: Option<DateTime>, limit: usize) -> Result<Vec<InAppNotification>, Box<dyn Error + Send + Sync>> {
        let (device, notifications) = self.repository.get_device_notifications(device_id.to_string(), since, limit).await?;
        let localizer = LanguageLocalizer::new_with_language(device.locale.as_ref());
        Ok(notifications.into_iter().filter_map(|notification| map_notification(notification, &localizer)).collect())
    }

    pub async fn mark_as_read(&self, wallet_id: i32) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.set_notifications_read(wallet_id).await?)
    }
}
