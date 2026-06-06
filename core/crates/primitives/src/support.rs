use crate::UInt64;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SupportMessageDeliveryStatus {
    Sending,
    Sent,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct SupportAgent {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum SupportMessageSender {
    User,
    Agent(SupportAgent),
}

impl SupportMessageSender {
    pub fn is_user(&self) -> bool {
        match self {
            Self::User => true,
            Self::Agent(_) => false,
        }
    }

    pub fn is_agent(&self) -> bool {
        match self {
            Self::User => false,
            Self::Agent(_) => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct SupportMessageImage {
    pub id: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<UInt64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct SupportMessage {
    pub id: String,
    pub content: String,
    pub sender: SupportMessageSender,
    pub status: SupportMessageDeliveryStatus,
    pub created_at: DateTime<Utc>,
    pub images: Vec<SupportMessageImage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct SupportMessageInput {
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SupportTypingStatus {
    On,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum SupportAction {
    Typing(SupportTypingStatus),
    LastSeen,
}
