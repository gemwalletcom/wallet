use chrono::{DateTime, Utc};
use primitives::{Device, SupportAgent, SupportMessage, SupportMessageDeliveryStatus, SupportMessageImage, SupportMessageSender, SupportTypingStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::constants::{
    CHATWOOT_CONTENT_TYPE_TEXT, CHATWOOT_DELIVERY_STATUS_DELIVERED, CHATWOOT_DELIVERY_STATUS_READ, CHATWOOT_DELIVERY_STATUS_SENT, CHATWOOT_FILE_TYPE_IMAGE,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum MessageType {
    Incoming,
    Outgoing,
}

impl From<i32> for MessageType {
    fn from(value: i32) -> Self {
        match value {
            1 => MessageType::Outgoing,
            _ => MessageType::Incoming,
        }
    }
}

impl From<MessageType> for i32 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::Incoming => 0,
            MessageType::Outgoing => 1,
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
    pub avatar_url: Option<String>,
    pub thumbnail: Option<String>,
    pub custom_attributes: Option<CustomAttributes>,
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
            SupportMessageDeliveryStatus::Sent,
            self.created_at.as_ref()?.datetime()?,
            &self.attachments,
        )
    }
}

impl Message {
    pub(crate) fn support_message(&self) -> Option<SupportMessage> {
        let sender = match &self.message_type {
            MessageType::Incoming => SupportMessageSender::User,
            MessageType::Outgoing => SupportMessageSender::Agent(self.sender.as_ref()?.support_agent()?),
        };

        support_message(
            self.id,
            self.content.as_deref(),
            self.content_type.as_deref(),
            self.private,
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
            thumbnail_url: self.thumb_url.clone().filter(|value| !value.is_empty()),
            file_name: self.fallback_title.clone().filter(|value| !value.is_empty()),
            file_size: self.file_size,
            width: self.width,
            height: self.height,
        })
    }
}

impl Sender {
    fn support_agent(&self) -> Option<SupportAgent> {
        let name = self.name.clone()?;
        Some(SupportAgent {
            name,
            avatar_url: self.avatar_url.clone().or_else(|| self.thumbnail.clone()).filter(|value| !value.is_empty()),
        })
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
    pub(crate) name: String,
    pub(crate) custom_attributes: HashMap<String, String>,
}

impl ChatwootContactUpdate {
    pub(crate) fn new(device: &Device) -> Self {
        Self {
            identifier: device.id.clone(),
            name: device.model.clone(),
            custom_attributes: HashMap::from([
                ("device_id".to_string(), device.id.clone()),
                ("platform".to_string(), format!("{}: {}", device.platform_store.as_ref(), device.os)),
                ("device".to_string(), device.model.clone()),
                ("version".to_string(), device.version.clone()),
                ("locale".to_string(), format!("{}: {}", device.locale, device.currency)),
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
        Self {
            message: ChatwootMessageData { content },
        }
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
        Self {
            typing_status: typing_status.to_string(),
        }
    }
}

pub(crate) fn support_messages(messages: &[Message]) -> Vec<SupportMessage> {
    messages.iter().filter_map(Message::support_message).collect()
}

fn support_message(
    id: i64,
    content: Option<&str>,
    content_type: Option<&str>,
    private: Option<bool>,
    sender: SupportMessageSender,
    delivery_status: SupportMessageDeliveryStatus,
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
        delivery_status,
        created_at,
        images,
    })
}

fn support_images(attachments: &[Attachment]) -> Vec<SupportMessageImage> {
    attachments.iter().filter_map(Attachment::support_image).collect()
}

fn support_delivery_status(status: Option<&str>) -> SupportMessageDeliveryStatus {
    match status {
        Some(CHATWOOT_DELIVERY_STATUS_SENT) | Some(CHATWOOT_DELIVERY_STATUS_DELIVERED) | Some(CHATWOOT_DELIVERY_STATUS_READ) | None => SupportMessageDeliveryStatus::Sent,
        Some(_) => SupportMessageDeliveryStatus::Failed,
    }
}

fn datetime_from_unix_timestamp(value: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp(value, 0)
}
