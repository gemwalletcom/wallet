use super::rules;
use primitives::{SupportMessage, SupportMessageSender};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSupportChatGroup {
    pub sender: SupportMessageSender,
    pub messages: Vec<SupportMessage>,
}

#[uniffi::export]
pub fn support_chat_groups(messages: Vec<SupportMessage>) -> Vec<GemSupportChatGroup> {
    rules::chat_groups(messages)
}
