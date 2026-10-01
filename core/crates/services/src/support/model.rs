use std::collections::HashMap;

use chrono::{DateTime, Utc};
use primitives::{Device, OptionStringExt, SupportAgent, SupportMessage, SupportMessageImage, SupportMessageSender, SupportMessageStatus, SupportTyping, SupportTypingStatus};
use serde::{Deserialize, Serialize};

use super::constants::{CHATWOOT_CONTENT_TYPE_TEXT, CHATWOOT_DELIVERY_STATUS_DELIVERED, CHATWOOT_DELIVERY_STATUS_READ, CHATWOOT_DELIVERY_STATUS_SENT, CHATWOOT_FILE_TYPE_IMAGE};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum MessageType {
    Incoming,
    Outgoing,
    Other(i32),
}

impl From<i32> for MessageType {
    fn from(value: i32) -> Self {
        match value {
            0 => MessageType::Incoming,
            1 => MessageType::Outgoing,
            other => MessageType::Other(other),
        }
    }
}

impl From<MessageType> for i32 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::Incoming => 0,
            MessageType::Outgoing => 1,
            MessageType::Other(value) => value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatwootDateTime {
    UnixTimestamp(i64),
    Rfc3339(DateTime<Utc>),
}

impl ChatwootDateTime {
    fn datetime(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::UnixTimestamp(value) => datetime_from_unix_timestamp(*value),
            Self::Rfc3339(value) => Some(*value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootWebhookPayload {
    pub event: String,
    pub id: Option<i64>,
    pub message_type: Option<String>,
    pub private: Option<bool>,
    pub unread_count: Option<i32>,
    pub conversation: Option<Conversation>,
    pub meta: Option<Meta>,
    pub content: Option<String>,
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub contact_last_seen_at: Option<i64>,
    pub last_activity_at: Option<i64>,
    #[serde(default)]
    pub created_at: Option<ChatwootDateTime>,
    pub sender: Option<Sender>,
    pub user: Option<WebhookUser>,
    pub is_private: Option<bool>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Option<i64>,
    pub meta: Meta,
    pub status: Option<String>,
    pub unread_count: Option<i32>,
    pub contact_last_seen_at: Option<i64>,
    pub last_activity_at: Option<i64>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub conversation_id: Option<i64>,
    pub content: Option<String>,
    pub message_type: MessageType,
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub private: Option<bool>,
    pub created_at: i64,
    pub sender: Option<Sender>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub sender: Sender,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributes {
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    pub name: Option<String>,
    pub custom_attributes: Option<CustomAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookUser {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub user_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: i64,
    pub file_type: Option<String>,
    pub data_url: Option<String>,
    pub thumb_url: Option<String>,
    pub fallback_title: Option<String>,
    pub file_size: Option<u64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl ChatwootWebhookPayload {
    pub fn get_device_id(&self) -> Option<String> {
        let attrs = self.conversation.as_ref().map(|c| &c.meta).or(self.meta.as_ref())?.sender.custom_attributes.as_ref()?;
        attrs.device_id.clone()
    }

    pub fn is_public_outgoing_message(&self) -> bool {
        self.message_type.as_deref() == Some("outgoing") && self.private == Some(false)
    }

    pub fn support_message(&self) -> Option<SupportMessage> {
        let sender = match self.message_type.as_deref()? {
            "incoming" => SupportMessageSender::User,
            "outgoing" => SupportMessageSender::Agent(self.sender.as_ref()?.support_agent()?),
            _ => return None,
        };

        support_message(
            self.id?,
            self.content.as_deref(),
            self.content_type.as_deref(),
            self.private,
            sender,
            SupportMessageStatus::Sent,
            self.created_at.as_ref()?.datetime()?,
            &self.attachments,
        )
    }

    pub fn support_typing(&self, status: SupportTypingStatus) -> Option<SupportTyping> {
        if self.is_private == Some(true) {
            return None;
        }
        let agent = self.user.as_ref()?.support_agent()?;
        Some(SupportTyping { status, agent })
    }
}

impl WebhookUser {
    fn support_agent(&self) -> Option<SupportAgent> {
        match self.user_type.as_deref() {
            Some("user") | Some("agent_bot") => {}
            _ => return None,
        }
        let name = self.name.clone()?;
        Some(SupportAgent { name })
    }
}

impl Message {
    pub(crate) fn support_message(&self) -> Option<SupportMessage> {
        self.map_support_message(self.private)
    }

    pub(crate) fn support_public_message(&self) -> Option<SupportMessage> {
        self.map_support_message(self.private.or(Some(false)))
    }

    fn map_support_message(&self, private: Option<bool>) -> Option<SupportMessage> {
        let sender = match &self.message_type {
            MessageType::Incoming => SupportMessageSender::User,
            MessageType::Outgoing => SupportMessageSender::Agent(self.sender.as_ref()?.support_agent()?),
            MessageType::Other(_) => return None,
        };

        support_message(
            self.id,
            self.content.as_deref(),
            self.content_type.as_deref(),
            private,
            sender,
            support_delivery_status(self.status.as_deref()),
            datetime_from_unix_timestamp(self.created_at)?,
            &self.attachments,
        )
    }
}

impl Attachment {
    fn support_image(&self) -> Option<SupportMessageImage> {
        if self.file_type.as_deref() != Some(CHATWOOT_FILE_TYPE_IMAGE) {
            return None;
        }

        Some(SupportMessageImage {
            id: self.id.to_string(),
            url: self.data_url.clone()?,
            thumbnail_url: self.thumb_url.clone().non_empty(),
            file_name: self.fallback_title.clone().non_empty(),
            file_size: self.file_size,
            width: self.width,
            height: self.height,
        })
    }
}

impl Sender {
    fn support_agent(&self) -> Option<SupportAgent> {
        let name = self.name.clone()?;
        Some(SupportAgent { name })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatwootSession {
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChatwootConfigResponse {
    pub(crate) website_channel_config: ChatwootWebsiteChannelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChatwootWebsiteChannelConfig {
    pub(crate) auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChatwootContactResponse {
    pub(crate) widget_auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChatwootMessagesResponse {
    pub(crate) payload: Vec<Message>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChatwootContactUpdate {
    pub(crate) identifier: String,
    pub(crate) custom_attributes: HashMap<String, String>,
}

impl ChatwootContactUpdate {
    pub(crate) fn new(device: &Device) -> Self {
        Self {
            identifier: device.id.clone(),
            custom_attributes: HashMap::from([
                ("device_id".to_string(), device.id.clone()),
                ("app".to_string(), format!("Version {}, {}, {}", device.version, device.os, device.platform_store.name())),
                ("device".to_string(), device.model.clone()),
                ("locale".to_string(), format!("{}, {}", device.locale.as_ref(), device.currency)),
            ]),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChatwootMessageInput {
    pub(crate) message: ChatwootMessageData,
}

impl ChatwootMessageInput {
    pub(crate) fn new(content: String) -> Self {
        Self { message: ChatwootMessageData { content } }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChatwootMessageData {
    pub(crate) content: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChatwootTypingInput {
    pub(crate) typing_status: String,
}

impl ChatwootTypingInput {
    pub(crate) fn new(status: SupportTypingStatus) -> Self {
        let typing_status = match status {
            SupportTypingStatus::On => "on",
            SupportTypingStatus::Off => "off",
        };
        Self { typing_status: typing_status.to_string() }
    }
}

pub(crate) fn support_public_messages(messages: &[Message]) -> Vec<SupportMessage> {
    messages.iter().filter_map(Message::support_public_message).collect()
}

fn support_message(
    id: i64,
    content: Option<&str>,
    content_type: Option<&str>,
    private: Option<bool>,
    sender: SupportMessageSender,
    status: SupportMessageStatus,
    created_at: DateTime<Utc>,
    attachments: &[Attachment],
) -> Option<SupportMessage> {
    if private != Some(false) {
        return None;
    }

    let images = support_images(attachments);
    if content_type.is_some_and(|content_type| content_type != CHATWOOT_CONTENT_TYPE_TEXT) && images.is_empty() {
        return None;
    }

    let content = content.unwrap_or_default().to_string();
    if content.is_empty() && images.is_empty() {
        return None;
    }

    Some(SupportMessage {
        id: id.to_string(),
        content,
        sender,
        status,
        created_at,
        images,
    })
}

fn support_images(attachments: &[Attachment]) -> Vec<SupportMessageImage> {
    attachments.iter().filter_map(Attachment::support_image).collect()
}

fn support_delivery_status(status: Option<&str>) -> SupportMessageStatus {
    match status {
        Some(CHATWOOT_DELIVERY_STATUS_SENT) | Some(CHATWOOT_DELIVERY_STATUS_DELIVERED) | Some(CHATWOOT_DELIVERY_STATUS_READ) | None => SupportMessageStatus::Sent,
        Some(_) => SupportMessageStatus::Failed,
    }
}

fn datetime_from_unix_timestamp(value: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp(value, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::currency::Currency;
    use primitives::{Device, DeviceLocale, PlatformStore, SupportAgent, SupportMessageSender, SupportMessageStatus};

    #[test]
    fn test_chatwoot_contact_update_attributes() {
        let device = Device {
            platform_store: PlatformStore::ApkUniversal,
            os: "Android 11".to_string(),
            model: "Xiaomi Redmi Note 8 Pro".to_string(),
            locale: DeviceLocale::FA,
            version: "2.93".to_string(),
            currency: Currency::USD,
            ..Device::mock()
        };

        let update = ChatwootContactUpdate::new(&device);

        assert_eq!(update.identifier, "test-device-id");
        assert_eq!(update.custom_attributes.get("device_id").map(String::as_str), Some("test-device-id"));
        assert_eq!(update.custom_attributes.get("app").map(String::as_str), Some("Version 2.93, Android 11, APK Universal"));
        assert_eq!(update.custom_attributes.get("device").map(String::as_str), Some("Xiaomi Redmi Note 8 Pro"));
        assert_eq!(update.custom_attributes.get("locale").map(String::as_str), Some("fa, USD"));
        assert_eq!(update.custom_attributes.len(), 4);
    }

    #[test]
    fn test_support_public_messages_maps_widget_messages_without_private() {
        let response: ChatwootMessagesResponse = serde_json::from_str(
            r#"{
                "payload": [{
                    "id": 1,
                    "content": "from agent",
                    "conversation_id": 2,
                    "message_type": 1,
                    "content_type": "text",
                    "created_at": 1766478193,
                    "sender": {"name": "Test Agent"}
                }]
            }"#,
        )
        .unwrap();

        let messages = support_public_messages(&response.payload);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "1");
        assert_eq!(messages[0].content, "from agent");
        assert_eq!(messages[0].sender, SupportMessageSender::mock_agent("Test Agent"));
        assert_eq!(messages[0].status, SupportMessageStatus::Sent);
    }

    #[test]
    fn test_support_public_messages_skips_template_messages() {
        let response: ChatwootMessagesResponse = serde_json::from_str(
            r#"{
                "payload": [{
                    "id": 1,
                    "content": "Give the team a way to reach you.",
                    "conversation_id": 2,
                    "message_type": 3,
                    "created_at": 1766478193
                }]
            }"#,
        )
        .unwrap();

        assert!(support_public_messages(&response.payload).is_empty());
    }

    #[test]
    fn test_support_typing() {
        let agent_on: ChatwootWebhookPayload = serde_json::from_str(include_str!("../../testdata/chatwoot_conversation_typing_on.json")).unwrap();

        let typing = agent_on.support_typing(SupportTypingStatus::On).unwrap();

        assert_eq!(typing.status, SupportTypingStatus::On);
        assert_eq!(typing.agent, SupportAgent { name: "Test Agent".to_string() });
        assert_eq!(agent_on.get_device_id(), Some("test-device-id".to_string()));

        let agent_bot: ChatwootWebhookPayload = serde_json::from_str(r#"{"event":"conversation_typing_on","is_private":false,"user":{"name":"Gemmy","type":"agent_bot"}}"#).unwrap();
        assert_eq!(agent_bot.support_typing(SupportTypingStatus::On).unwrap().agent, SupportAgent { name: "Gemmy".to_string() });

        let private: ChatwootWebhookPayload = serde_json::from_str(r#"{"event":"conversation_typing_on","is_private":true,"user":{"name":"Test Agent","type":"user"}}"#).unwrap();
        assert_eq!(private.support_typing(SupportTypingStatus::On), None);

        let contact: ChatwootWebhookPayload = serde_json::from_str(r#"{"event":"conversation_typing_on","is_private":false,"user":{"name":"test-user","type":"contact"}}"#).unwrap();
        assert_eq!(contact.support_typing(SupportTypingStatus::On), None);
    }

    #[test]
    fn test_support_messages_keep_missing_private_strict() {
        let message: Message = serde_json::from_str(
            r#"{
                "id": 1,
                "content": "from agent",
                "conversation_id": 2,
                "message_type": 1,
                "content_type": "text",
                "created_at": 1766478193,
                "sender": {"name": "Test Agent"}
            }"#,
        )
        .unwrap();

        assert_eq!(message.support_message(), None);
    }

    #[test]
    fn test_parse_device_id() {
        let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "conversation_updated", "meta": {"sender": {"custom_attributes": {"device_id": "test-device"}}}}"#).unwrap();
        assert_eq!(payload.get_device_id(), Some("test-device".to_string()));
    }

    #[test]
    fn test_parse_message_created_payload() {
        let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("../../testdata/chatwoot_message_created.json")).unwrap();
        assert_eq!(payload.event, "message_created");
        assert_eq!(payload.content, Some("from agent".to_string()));
        assert_eq!(payload.get_device_id(), Some("test-device-id".to_string()));
        assert!(payload.is_public_outgoing_message());
    }

    #[test]
    fn test_is_public_outgoing_message() {
        let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing", "private": false}"#).unwrap();
        assert!(payload.is_public_outgoing_message());

        let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing", "private": true}"#).unwrap();
        assert!(!payload.is_public_outgoing_message());

        let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "incoming", "private": false}"#).unwrap();
        assert!(!payload.is_public_outgoing_message());

        let payload: ChatwootWebhookPayload = serde_json::from_str(r#"{"event": "message_created", "message_type": "outgoing"}"#).unwrap();
        assert!(!payload.is_public_outgoing_message());
    }

    #[test]
    fn test_support_message_mapping() {
        let payload: ChatwootWebhookPayload = serde_json::from_str(include_str!("../../testdata/chatwoot_message_created.json")).unwrap();
        let message = payload.support_message().unwrap();
        assert_eq!(message.id, "1");
        assert_eq!(message.content, "from agent");
        assert!(message.images.is_empty());
        assert_eq!(message.sender, SupportMessageSender::mock_agent("Test Agent"));
        assert_eq!(message.status, SupportMessageStatus::Sent);
    }

    #[test]
    fn test_support_message_maps_image_attachment() {
        let payload: ChatwootWebhookPayload = serde_json::from_str(
            r#"{
                "event": "message_created",
                "id": 10,
                "conversation": {"id": 1, "meta": {"sender": {}}},
                "message_type": "outgoing",
                "private": false,
                "content": null,
                "content_type": "text",
                "created_at": "2025-12-23T08:23:13.554Z",
                "sender": {"name": "Test Agent"},
                "attachments": [{
                    "id": 7,
                    "file_type": "image",
                    "data_url": "https://support.gemwallet.com/image.png",
                    "thumb_url": "https://support.gemwallet.com/thumb.png",
                    "fallback_title": "proof.png",
                    "file_size": 1234,
                    "width": 640,
                    "height": 480
                }]
            }"#,
        )
        .unwrap();

        let message = payload.support_message().unwrap();
        assert_eq!(message.content, "");
        assert_eq!(message.images.len(), 1);
        let image = &message.images[0];
        assert_eq!(image.id, "7");
        assert_eq!(image.url, "https://support.gemwallet.com/image.png");
        assert_eq!(image.thumbnail_url.as_deref(), Some("https://support.gemwallet.com/thumb.png"));
        assert_eq!(image.file_name.as_deref(), Some("proof.png"));
        assert_eq!(image.file_size, Some(1234));
        assert_eq!(image.width, Some(640));
        assert_eq!(image.height, Some(480));
    }

    #[test]
    fn test_support_message_ignores_attachment_without_image() {
        let payload: ChatwootWebhookPayload = serde_json::from_str(
            r#"{
                "event": "message_created",
                "id": 10,
                "conversation": {"id": 1, "meta": {"sender": {}}},
                "message_type": "outgoing",
                "private": false,
                "content": null,
                "content_type": "text",
                "created_at": 1766478193,
                "sender": {"name": "Test Agent"},
                "attachments": [{
                    "id": 7,
                    "file_type": "file",
                    "data_url": "https://support.gemwallet.com/file.pdf"
                }]
            }"#,
        )
        .unwrap();

        assert!(payload.support_message().is_none());
    }
}
