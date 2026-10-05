use super::rules;
use crate::models::state::{GemListPhase, GemLoadState};
use crate::services::empty_state::{GemEmptyStateKind, empty_state};
use primitives::SupportMessage;
use support::SupportMessageDisplayContent;

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

#[uniffi::export]
pub fn support_list_phase(messages: Vec<SupportMessage>, state: GemLoadState) -> GemListPhase {
    GemListPhase::new(state, !messages.is_empty(), empty_state(GemEmptyStateKind::Support))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::error::GemServiceError;

    #[test]
    fn test_a_chat_shows_its_messages_before_a_failed_sync_and_its_empty_state_without() {
        let error = GemServiceError::Gateway { msg: "offline".to_string() };

        assert_eq!(support_list_phase(vec![SupportMessage::mock("1", 1)], GemLoadState::Error { error: error.clone() }), GemListPhase::Rows);
        assert_eq!(support_list_phase(vec![], GemLoadState::Error { error: error.clone() }), GemListPhase::Error { error });
        assert_eq!(
            support_list_phase(vec![], GemLoadState::Loading),
            GemListPhase::Empty {
                state: empty_state(GemEmptyStateKind::Support)
            }
        );
    }
}
