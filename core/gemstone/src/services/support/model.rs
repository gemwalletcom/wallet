use super::rules;
use primitives::{SupportMessage, SupportMessageSender};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSupportChatGroup {
    pub sender: SupportMessageSender,
    pub messages: Vec<SupportMessage>,
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

#[uniffi::export]
pub fn support_message_outcome(message: SupportMessage) -> GemSupportMessageOutcome {
    rules::message_outcome(&message)
}
