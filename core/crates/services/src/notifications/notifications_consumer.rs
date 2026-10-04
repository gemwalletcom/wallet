use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use gem_tracing::info_with_fields;
use pusher::PusherClient;
use streamer::{NotificationsFailedPayload, NotificationsPayload, StreamProducerQueue, consumer::MessageConsumer};

pub struct NotificationsConsumer {
    pub pusher: PusherClient,
    pub stream_producer: Arc<dyn StreamProducerQueue>,
}

impl NotificationsConsumer {
    pub fn new(pusher: PusherClient, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
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
