use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use streamer::{RewardsNotificationPayload, StreamProducerQueue, consumer::MessageConsumer};

use super::repository::Repository;

pub struct RewardsConsumer {
    repository: Arc<dyn Repository>,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

#[async_trait]
impl MessageConsumer<RewardsNotificationPayload, usize> for RewardsConsumer {
    async fn should_consume(&self, _payload: &RewardsNotificationPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn consume(&self, payload: RewardsNotificationPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let notifications = self.repository.event_notifications(payload.event_id).await?;
        let count = notifications.len();
        self.stream_producer.publish_in_app_notifications(notifications).await?;
        Ok(count)
    }
}

impl RewardsConsumer {
    pub(crate) fn new(repository: Arc<dyn Repository>, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self { repository, stream_producer }
    }
}
