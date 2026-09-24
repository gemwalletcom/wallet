use super::rules;
use crate::support::SupportMessageDisplayContent;
use primitives::{SupportMessage, SupportMessageSender};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSupportMessageRow {
    pub message: SupportMessage,
    pub content: SupportMessageDisplayContent,
    pub outcome: GemSupportMessageOutcome,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSupportChatGroup {
    pub sender: SupportMessageSender,
    pub rows: Vec<GemSupportMessageRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSupportMessageOutcome {
    Sending,
    Sent,
    Failed { can_retry: bool },
}

#[uniffi::export]
pub fn support_chat_groups(messages: Vec<SupportMessage>) -> Vec<GemSupportChatGroup> {
    rules::chat_groups(messages)
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemSupportAttachmentLimits {
    pub max_dimension: u32,
    pub jpeg_quality: u32,
}
