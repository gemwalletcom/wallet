use std::error::Error;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use in_app_notifications::map_notification;
use localizer::LanguageLocalizer;
use primitives::InAppNotification;

use super::repository::Repository;

#[derive(Clone)]
pub struct NotificationsClient {
    repository: Arc<dyn Repository>,
}

impl NotificationsClient {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn get_notifications(&self, device_id: &str, from_timestamp: Option<u64>, limit: usize) -> Result<Vec<InAppNotification>, Box<dyn Error + Send + Sync>> {
        let from_datetime = from_timestamp.and_then(|ts| DateTime::<Utc>::from_timestamp(ts as i64, 0).map(|dt| dt.naive_utc()));
        let (device, notifications) = self.repository.device_notifications(device_id.to_string(), from_datetime, limit).await?;
        let localizer = LanguageLocalizer::new_with_language(device.locale.as_ref());
        Ok(notifications.into_iter().filter_map(|notification| map_notification(notification, &localizer)).collect())
    }

    pub async fn mark_all_as_read(&self, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.mark_all_as_read(device_id.to_string()).await?)
    }
}
