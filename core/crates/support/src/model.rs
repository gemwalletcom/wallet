use chrono::{DateTime, Utc};
use primitives::{
    Device, SupportAgent, SupportConversation, SupportConversationStatus, SupportMessage, SupportMessageDeliveryStatus, SupportMessageImage, SupportMessageSender,
    SupportTypingStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::constants::{
    CHATWOOT_CONTENT_TYPE_TEXT, CHATWOOT_DELIVERY_STATUS_DELIVERED, CHATWOOT_DELIVERY_STATUS_READ, CHATWOOT_DELIVERY_STATUS_SENT, CHATWOOT_FILE_TYPE_IMAGE,
    CHATWOOT_STATUS_RESOLVED,
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

    fn conversation_id(&self) -> Option<i64> {
        self.conversation.as_ref().and_then(|c| c.id)
    }

    fn messages(&self) -> &[Message] {
        if !self.messages.is_empty() {
            &self.messages
        } else if let Some(conversation) = &self.conversation {
            &conversation.messages
        } else {
            &[]
        }
    }

    pub fn support_message(&self) -> Option<SupportMessage> {
        let sender = match self.message_type.as_deref()? {
            "incoming" => SupportMessageSender::User,
            "outgoing" => SupportMessageSender::Agent(self.sender.as_ref()?.support_agent()?),
            _ => return None,
        };

        support_message(
            self.id?,
            self.conversation_id()?,
            self.content.as_deref(),
            self.content_type.as_deref(),
            self.private,
            sender,
            SupportMessageDeliveryStatus::Sent,
            self.created_at.as_ref()?.datetime()?,
            &self.attachments,
        )
    }

    pub fn support_conversation(&self) -> Option<SupportConversation> {
        if let Some(conversation) = &self.conversation {
            return conversation.support_conversation();
        }

        map_support_conversation(
            self.id?,
            self.status.as_deref(),
            self.messages(),
            self.unread_count,
            self.contact_last_seen_at,
            self.last_activity_at
                .and_then(datetime_from_unix_timestamp)
                .or_else(|| self.created_at.as_ref().and_then(ChatwootDateTime::datetime)),
        )
    }
}

impl Conversation {
    fn support_conversation(&self) -> Option<SupportConversation> {
        map_support_conversation(
            self.id?,
            self.status.as_deref(),
            &self.messages,
            self.unread_count,
            self.contact_last_seen_at,
            self.last_activity_at.and_then(datetime_from_unix_timestamp),
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
            self.conversation_id?,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChatwootConversationResponse {
    pub(crate) id: Option<i64>,
    pub(crate) status: Option<String>,
    pub(crate) contact_last_seen_at: Option<i64>,
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
                ("platform".to_string(), device.platform.as_ref().to_string()),
                ("os".to_string(), device.os.clone()),
                ("device".to_string(), device.model.clone()),
                ("app_version".to_string(), device.version.clone()),
                ("currency".to_string(), device.currency.clone()),
            ]),
        }
    }
}

impl ChatwootConversationResponse {
    pub(crate) fn support_conversation(&self, messages: &[SupportMessage]) -> Option<SupportConversation> {
        support_conversation(
            self.id?.to_string(),
            self.status.as_deref(),
            messages,
            None,
            self.contact_last_seen_at,
            messages
                .last()
                .map(|message| message.created_at)
                .or_else(|| self.contact_last_seen_at.and_then(datetime_from_unix_timestamp)),
        )
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
    conversation_id: i64,
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
        conversation_id: conversation_id.to_string(),
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

fn map_support_conversation(
    id: i64,
    status: Option<&str>,
    messages: &[Message],
    unread_count_value: Option<i32>,
    contact_last_seen_at: Option<i64>,
    explicit_last_activity_at: Option<DateTime<Utc>>,
) -> Option<SupportConversation> {
    let messages = support_messages(messages);
    support_conversation(
        id.to_string(),
        status,
        &messages,
        unread_count_value,
        contact_last_seen_at,
        explicit_last_activity_at.or_else(|| messages.last().map(|message| message.created_at)),
    )
}

fn support_conversation(
    id: String,
    status: Option<&str>,
    messages: &[SupportMessage],
    unread_count_value: Option<i32>,
    contact_last_seen_at: Option<i64>,
    last_activity_at: Option<DateTime<Utc>>,
) -> Option<SupportConversation> {
    let first_message = messages
        .iter()
        .find(|message| message.sender.is_user() && !message.content.is_empty())
        .map(|message| message.content.clone());
    let last_message = messages.iter().rev().find(|message| !message.content.is_empty()).map(|message| message.content.clone());

    Some(SupportConversation {
        id,
        status: support_conversation_status(status),
        first_message,
        last_message,
        last_activity_at: last_activity_at?,
        unread_count: unread_count_value.unwrap_or_else(|| unread_count(messages, contact_last_seen_at)),
    })
}

fn support_conversation_status(status: Option<&str>) -> SupportConversationStatus {
    match status {
        Some(CHATWOOT_STATUS_RESOLVED) => SupportConversationStatus::Resolved,
        _ => SupportConversationStatus::Open,
    }
}

fn support_delivery_status(status: Option<&str>) -> SupportMessageDeliveryStatus {
    match status {
        Some(CHATWOOT_DELIVERY_STATUS_SENT) | Some(CHATWOOT_DELIVERY_STATUS_DELIVERED) | Some(CHATWOOT_DELIVERY_STATUS_READ) | None => SupportMessageDeliveryStatus::Sent,
        Some(_) => SupportMessageDeliveryStatus::Failed,
    }
}

fn unread_count(messages: &[SupportMessage], contact_last_seen_at: Option<i64>) -> i32 {
    let Some(contact_last_seen_at) = contact_last_seen_at else {
        return 0;
    };
    messages
        .iter()
        .filter(|message| message.sender.is_agent() && message.created_at.timestamp() > contact_last_seen_at)
        .count() as i32
}

fn datetime_from_unix_timestamp(value: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp(value, 0)
}
