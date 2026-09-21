use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{SupportMessage, SupportTyping};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemSupportStore: Send + Sync {
    async fn save_messages(&self, messages: Vec<SupportMessage>) -> Result<(), GemServiceError>;
    async fn save_message(&self, id: String, message: SupportMessage) -> Result<(), GemServiceError>;
    async fn fail_pending_messages(&self, except_ids: Vec<String>) -> Result<(), GemServiceError>;
    fn update_typing(&self, typing: SupportTyping) -> Result<(), GemServiceError>;
    fn clear_typing(&self) -> Result<(), GemServiceError>;
}
