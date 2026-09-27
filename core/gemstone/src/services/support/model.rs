use super::rules;
use crate::support::SupportMessageDisplayContent;
use primitives::SupportMessage;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSupportMessageRow {
    pub message: SupportMessage,
    pub content: SupportMessageDisplayContent,
    pub side: GemSupportBubbleSide,
    pub outcome: GemSupportMessageOutcome,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSupportChatGroup {
    pub side: GemSupportBubbleSide,
    pub rows: Vec<GemSupportMessageRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSupportBubbleSide {
    Outgoing,
    Incoming,
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
