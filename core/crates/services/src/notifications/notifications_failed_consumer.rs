use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use storage::{DeviceFilter, DeviceUpdate};
use streamer::{NotificationsFailedPayload, consumer::MessageConsumer};

use super::repository::Repository;

pub struct NotificationsFailedConsumer {
    pub(crate) repository: Arc<dyn Repository>,
}

impl NotificationsFailedConsumer {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl MessageConsumer<NotificationsFailedPayload, usize> for NotificationsFailedConsumer {
    async fn should_consume(&self, _payload: &NotificationsFailedPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn consume(&self, payload: NotificationsFailedPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let device_ids: Vec<String> = payload.failures.iter().filter(|f| f.error.is_device_invalid()).map(|f| f.notification.device_id.clone()).collect();

        if device_ids.is_empty() {
            return Ok(0);
        }

        Ok(self.repository.update_devices(vec![DeviceFilter::Ids(device_ids)], vec![DeviceUpdate::IsPushEnabled(false)]).await?)
    }
}

#[cfg(test)]
mod tests {
    use primitives::Device;
    use push_notification::{FailedNotification, GorushNotification, PushErrorLog, PushNotification, PushNotificationTypes};

    use super::*;
    use crate::testkit::MemoryNotificationsRepository;

    fn failure(device_id: &str, error: &str) -> FailedNotification {
        let device = Device { id: device_id.to_string(), ..Device::mock() };
        let data = PushNotification {
            notification_type: PushNotificationTypes::Test,
            data: None,
        };
        FailedNotification {
            notification: GorushNotification::from_device(device, "title".to_string(), "message".to_string(), data).unwrap(),
            error: PushErrorLog {
                token: "token".to_string(),
                error: error.to_string(),
            },
        }
    }

    #[tokio::test]
    async fn test_disables_push_only_for_invalid_tokens() {
        let repository = Arc::new(MemoryNotificationsRepository::default());
        let consumer = NotificationsFailedConsumer::new(repository.clone());

        let count = consumer
            .consume(NotificationsFailedPayload::new(vec![failure("invalid", "BadDeviceToken"), failure("throttled", "TooManyRequests")]))
            .await
            .unwrap();

        assert_eq!(count, 1);
        let updates = repository.device_updates();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0], (vec![DeviceFilter::Ids(vec!["invalid".to_string()])], vec![DeviceUpdate::IsPushEnabled(false)]));
    }
}
