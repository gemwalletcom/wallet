use crate::{
    ChatwootWebhookPayload,
    constants::{EVENT_CONVERSATION_TYPING_OFF, EVENT_CONVERSATION_TYPING_ON, EVENT_MESSAGE_CREATED},
    markdown_plain_text,
};
use cacher::CacherClient;
use localizer::LanguageLocalizer;
use primitives::{
    Device, GorushNotification, PushNotification, PushNotificationTypes, StreamEvent, SupportMessage, SupportStreamEvent, SupportTypingStatus, device_stream_channel,
    push_notification::PushNotificationSupport,
};
use std::error::Error;
use storage::database::devices::DevicesStore;
use storage::{Database, OptionalExtension};
use streamer::{NotificationsPayload, StreamProducer, StreamProducerQueue};

#[derive(Debug, Default)]
pub struct SupportWebhookResult {
    pub notifications: usize,
    pub stream_events: usize,
}

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

    pub async fn process_webhook(&self, device: &Device, payload: &ChatwootWebhookPayload) -> Result<SupportWebhookResult, Box<dyn Error + Send + Sync>> {
        match payload.event.as_str() {
            EVENT_MESSAGE_CREATED => self.process_message_created(device, payload).await,
            EVENT_CONVERSATION_TYPING_ON => self.process_typing(device, payload, SupportTypingStatus::On).await,
            EVENT_CONVERSATION_TYPING_OFF => self.process_typing(device, payload, SupportTypingStatus::Off).await,
            _ => Ok(SupportWebhookResult::default()),
        }
    }

    async fn process_message_created(&self, device: &Device, payload: &ChatwootWebhookPayload) -> Result<SupportWebhookResult, Box<dyn Error + Send + Sync>> {
        let notifications = if let Some(notification) = Self::build_notification(device, payload) {
            self.stream_producer.publish_notifications_support(NotificationsPayload::new(vec![notification])).await?;
            1
        } else {
            0
        };

        let stream_events = self.publish_stream_message(device, payload).await?;

        Ok(SupportWebhookResult { notifications, stream_events })
    }

    async fn process_typing(&self, device: &Device, payload: &ChatwootWebhookPayload, status: SupportTypingStatus) -> Result<SupportWebhookResult, Box<dyn Error + Send + Sync>> {
        let Some(typing) = payload.support_typing(status) else {
            return Ok(SupportWebhookResult::default());
        };

        self.publish_event(device, StreamEvent::Support(SupportStreamEvent::Typing(typing))).await?;
        Ok(SupportWebhookResult { notifications: 0, stream_events: 1 })
    }

    fn build_notification(device: &Device, payload: &ChatwootWebhookPayload) -> Option<GorushNotification> {
        if !payload.is_public_outgoing_message() {
            return None;
        }

        let title = LanguageLocalizer::new_with_language(device.locale.as_str()).notification_support_new_message_title();
        let message = markdown_plain_text(payload.content.as_deref().unwrap_or_default());
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
        if !Self::should_publish_stream_message(&message) {
            return Ok(0);
        }

        self.publish_event(device, StreamEvent::Support(SupportStreamEvent::Message(message))).await?;
        Ok(1)
    }

    fn should_publish_stream_message(message: &SupportMessage) -> bool {
        message.sender.is_agent()
    }

    async fn publish_event(&self, device: &Device, event: StreamEvent) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.publish(&device_stream_channel(&device.id), &event).await
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

    #[test]
    fn test_user_message_is_not_streamed() {
        let payload: ChatwootWebhookPayload =
            serde_json::from_str(r#"{"id":1,"conversation":{"id":2,"meta":{"sender":{"custom_attributes":{"device_id":"test-device"}}}},"event":"message_created","message_type":"incoming","private":false,"content":"from user","content_type":"text","created_at":1780601365}"#).unwrap();

        let message = payload.support_message().unwrap();

        assert!(message.sender.is_user());
        assert!(!SupportClient::should_publish_stream_message(&message));
    }
}
