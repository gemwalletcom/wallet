use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use gem_tracing::info_with_fields;
use pusher::PushProvider;
use streamer::{NotificationsFailedPayload, NotificationsPayload, StreamProducerQueue, consumer::MessageConsumer};

pub struct NotificationsConsumer {
    pub pusher: Arc<dyn PushProvider>,
    pub stream_producer: Arc<dyn StreamProducerQueue>,
}

impl NotificationsConsumer {
    pub fn new(pusher: Arc<dyn PushProvider>, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self { pusher, stream_producer }
    }
}

#[async_trait]
impl MessageConsumer<NotificationsPayload, usize> for NotificationsConsumer {
    async fn should_consume(&self, _payload: &NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn consume(&self, payload: NotificationsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        for notification in &payload.notifications {
            info_with_fields!(
                "send push notification",
                device_id = notification.device_id.as_str(),
                notification_type = notification.data.notification_type.as_ref(),
                title = notification.title.as_str(),
                message = notification.message.as_str()
            );
        }

        let result = self.pusher.push_notifications(payload.notifications).await?;
        let counts = result.response.counts as usize;
        let success = &result.response.success;
        let logs = &result.response.logs;

        info_with_fields!("gorush response", counts = counts, success = success.as_str(), logs = format!("{:?}", logs));

        let failures = result.failures();

        if !failures.is_empty() {
            info_with_fields!(
                "push failures",
                count = failures.len(),
                failures = format!("{:?}", failures.iter().map(|f| (&f.notification.device_id, &f.error.error)).collect::<Vec<_>>())
            );
            self.stream_producer.publish_notifications_failed(NotificationsFailedPayload::new(failures)).await?;
        }

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use primitives::Device;
    use push_notification::{GorushNotification, PushErrorLog, PushNotification, PushNotificationTypes};
    use streamer::QueueName;

    use super::*;
    use crate::testkit::{RecordingPushProvider, RecordingStreamProducer};

    fn payload() -> NotificationsPayload {
        let data = PushNotification {
            notification_type: PushNotificationTypes::Test,
            data: None,
        };
        NotificationsPayload::new(GorushNotification::from_device(Device::mock(), "title".to_string(), "message".to_string(), data).into_iter().collect())
    }

    #[tokio::test]
    async fn test_consume_publishes_failed_tokens() {
        let pusher = Arc::new(RecordingPushProvider::new(vec![PushErrorLog {
            token: "test-token-123".to_string(),
            error: "BadDeviceToken".to_string(),
        }]));
        let producer = Arc::new(RecordingStreamProducer::new());
        let consumer = NotificationsConsumer::new(pusher.clone(), producer.clone());

        assert_eq!(consumer.consume(payload()).await.unwrap(), 1);

        assert_eq!(pusher.pushed().len(), 1);
        let published = producer.published();
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].0, QueueName::NotificationsFailed);
        assert_eq!(published[0].1["failures"][0]["notification"]["device_id"], "test-device-id");
        assert_eq!(published[0].1["failures"][0]["error"]["error"], "BadDeviceToken");
    }

    #[tokio::test]
    async fn test_consume_delivered_publishes_nothing() {
        let producer = Arc::new(RecordingStreamProducer::new());
        let consumer = NotificationsConsumer::new(Arc::new(RecordingPushProvider::new(vec![])), producer.clone());

        assert_eq!(consumer.consume(payload()).await.unwrap(), 1);
        assert!(producer.published().is_empty());
    }

    #[tokio::test]
    async fn test_consume_push_failure() {
        let producer = Arc::new(RecordingStreamProducer::new());
        let consumer = NotificationsConsumer::new(Arc::new(RecordingPushProvider::failing("gorush unavailable")), producer.clone());

        let error = consumer.consume(payload()).await.unwrap_err();

        assert!(error.to_string().contains("gorush unavailable"));
        assert!(producer.published().is_empty());
    }
}
