use std::error::Error;

use chrono::{DateTime, Utc};
use in_app_notifications::map_notification;
use localizer::LanguageLocalizer;
use primitives::InAppNotification;
use storage::{Database, DatabaseError, DevicesRepository, NotificationsRepository};

#[derive(Clone)]
pub struct NotificationsClient {
    database: Database,
}

impl NotificationsClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_notifications(&self, device_id: &str, from_timestamp: Option<u64>, limit: usize) -> Result<Vec<InAppNotification>, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        let from_datetime = from_timestamp.and_then(|ts| DateTime::<Utc>::from_timestamp(ts as i64, 0).map(|dt| dt.naive_utc()));
        let (device, notifications) = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let device = client.get_device(&device_id)?;
                let notifications = client.get_notifications_by_device_id(&device_id, from_datetime, limit)?;
                Ok((device, notifications))
            })
            .await?;
        let localizer = LanguageLocalizer::new_with_language(device.locale.as_ref());
        Ok(notifications.into_iter().filter_map(|notification| map_notification(notification, &localizer)).collect())
    }

    pub async fn mark_all_as_read(&self, device_id: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        Ok(self.database.run(move |client| client.mark_all_as_read(&device_id)).await?)
    }
}
