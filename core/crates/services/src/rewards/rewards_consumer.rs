use std::error::Error;

use async_trait::async_trait;
use primitives::{NotificationRewardsMetadata, NotificationType, RewardEvent, RewardEventType};
use storage::{Database, DatabaseClient, DatabaseError, RewardsRepository};
use streamer::{InAppNotificationPayload, RewardsNotificationPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

pub struct RewardsConsumer {
    database: Database,
    stream_producer: StreamProducer,
}

#[async_trait]
impl MessageConsumer<RewardsNotificationPayload, usize> for RewardsConsumer {
    async fn should_process(&self, _payload: &RewardsNotificationPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: RewardsNotificationPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let event_id = payload.event_id;
        let notifications = self
            .database
            .run(move |client| {
                let event = client.get_reward_event(event_id)?;
                create_in_app_notification_payloads(client, &event)
            })
            .await?;
        let count = notifications.len();
        self.stream_producer.publish_in_app_notifications(notifications).await?;
        Ok(count)
    }
}

impl RewardsConsumer {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }
}

fn create_in_app_notification_payloads(client: &mut DatabaseClient, event: &RewardEvent) -> Result<Vec<InAppNotificationPayload>, DatabaseError> {
    let metadata = NotificationRewardsMetadata {
        username: Some(event.username.clone()),
        points: (event.points > 0).then_some(event.points),
    };
    let metadata_value = serde_json::to_value(metadata).ok();

    match event.event {
        RewardEventType::CreateUsername => {
            let wallet_id = client.get_wallet_id_by_username(&event.username)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsCreateUsername, metadata_value)])
        }
        RewardEventType::InvitePending | RewardEventType::InviteNew => {
            let wallet_id = client.get_wallet_id_by_username(&event.username)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsInvite, metadata_value)])
        }
        RewardEventType::Joined => {
            let Some(referrer) = client.get_referrer_username(&event.username)? else {
                return Ok(vec![]);
            };
            let wallet_id = client.get_wallet_id_by_username(&referrer)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::ReferralJoined, metadata_value)])
        }
        RewardEventType::Enabled => {
            let wallet_id = client.get_wallet_id_by_username(&event.username)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsEnabled, metadata_value)])
        }
        RewardEventType::Disabled => {
            let wallet_id = client.get_wallet_id_by_username(&event.username)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsCodeDisabled, metadata_value)])
        }
        RewardEventType::Redeemed => {
            let wallet_id = client.get_wallet_id_by_username(&event.username)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsRedeemed, metadata_value)])
        }
    }
}
