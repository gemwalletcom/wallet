use crate::{
    ChatwootWebhookPayload,
    constants::{EVENT_CONVERSATION_STATUS_CHANGED, EVENT_CONVERSATION_UPDATED, EVENT_MESSAGE_CREATED},
};
use cacher::CacherClient;
use localizer::LanguageLocalizer;
use primitives::{
    Device, GorushNotification, PushNotification, PushNotificationTypes, StreamEvent, SupportStreamEvent, device_stream_channel, push_notification::PushNotificationSupport,
};
use std::error::Error;
use storage::database::devices::DevicesStore;
use storage::{Database, OptionalExtension};
use streamer::{NotificationsPayload, StreamProducer, StreamProducerQueue};

pub struct SupportClient {
    database: Database,
    stream_producer: StreamProducer,
    cacher: CacherClient,
}

impl SupportClient {
    pub fn new(database: Database, stream_producer: StreamProducer, cacher: CacherClient) -> Self {
        Self {
            database,
            stream_producer,
            cacher,
        }
    }

    pub fn get_device(&self, device_id: &str) -> Result<Option<Device>, Box<dyn Error + Send + Sync>> {
        Ok(DevicesStore::get_device(&mut self.database.client()?, device_id).optional()?.map(|d| d.as_primitive()))
    }

    pub async fn process_webhook(&self, device: &Device, payload: &ChatwootWebhookPayload) -> Result<(usize, usize), Box<dyn Error + Send + Sync>> {
        match payload.event.as_str() {
            EVENT_MESSAGE_CREATED => self.process_message_created(device, payload).await,
            EVENT_CONVERSATION_UPDATED | EVENT_CONVERSATION_STATUS_CHANGED => self.process_conversation_updated(device, payload).await,
            _ => Ok((0, 0)),
        }
    }

    async fn process_message_created(&self, device: &Device, payload: &ChatwootWebhookPayload) -> Result<(usize, usize), Box<dyn Error + Send + Sync>> {
        let notifications_count = if let Some(notification) = Self::build_notification(device, payload) {
            self.stream_producer.publish_notifications_support(NotificationsPayload::new(vec![notification])).await?;
            1
        } else {
            0
        };

        let stream_events_count = self.publish_stream_message(device, payload).await?;

        Ok((notifications_count, stream_events_count))
    }

    async fn process_conversation_updated(&self, device: &Device, payload: &ChatwootWebhookPayload) -> Result<(usize, usize), Box<dyn Error + Send + Sync>> {
        if let Some(conversation) = payload.support_conversation() {
            self.publish_stream_event(device, SupportStreamEvent::Conversation(conversation)).await?;
            Ok((0, 1))
        } else {
            Ok((0, 0))
        }
    }

    fn build_notification(device: &Device, payload: &ChatwootWebhookPayload) -> Option<GorushNotification> {
        if !payload.is_public_outgoing_message() {
            return None;
        }

        let title = LanguageLocalizer::new_with_language(device.locale.as_str()).notification_support_new_message_title();
        let message = payload.content.clone().unwrap_or_default();
        let data = PushNotification {
            notification_type: PushNotificationTypes::Support,
            data: serde_json::to_value(PushNotificationSupport {}).ok(),
        };

        GorushNotification::from_device(device.clone(), title, message, data)
    }

    async fn publish_stream_message(&self, device: &Device, payload: &ChatwootWebhookPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let Some(message) = payload.support_message() else {
            return Ok(0);
        };

        self.publish_stream_event(device, SupportStreamEvent::Message(message)).await?;
        Ok(1)
    }

    async fn publish_stream_event(&self, device: &Device, event: SupportStreamEvent) -> Result<(), Box<dyn Error + Send + Sync>> {
        let channel = device_stream_channel(&device.id);
        self.cacher.publish(&channel, &StreamEvent::Support(event)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_notification_message_created() {
        let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("../tests/testdata/chatwoot_message_created.json")).unwrap();

        let notification = SupportClient::build_notification(&Device::mock(), &payload);

        assert!(notification.is_some());
        assert_eq!(notification.unwrap().message, "from agent");
    }

    #[test]
    fn test_build_notification_conversation_updated() {
        let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("../tests/testdata/chatwoot_conversation_updated.json")).unwrap();

        let notification = SupportClient::build_notification(&Device::mock(), &payload);

        assert!(notification.is_none());
    }

    #[test]
    fn test_build_notification_private_message_created() {
        let payload: ChatwootWebhookPayload =
            serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing", "private": true, "content": "internal note"}"#).unwrap();

        let notification = SupportClient::build_notification(&Device::mock(), &payload);

        assert!(notification.is_none());
    }

    #[test]
    fn test_build_notification_missing_private_message_created() {
        let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing", "content": "unknown visibility"}"#).unwrap();

        let notification = SupportClient::build_notification(&Device::mock(), &payload);

        assert!(notification.is_none());
    }
}
