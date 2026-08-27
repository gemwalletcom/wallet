use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{SupportMessage, SupportTyping};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemSupportStore: Send + Sync {
    async fn save_messages(&self, messages: Vec<SupportMessage>) -> Result<(), GemServiceError>;
    async fn replace_message(&self, id: String, message: SupportMessage) -> Result<(), GemServiceError>;
    fn update_typing(&self, typing: SupportTyping) -> Result<(), GemServiceError>;
    fn clear_typing(&self) -> Result<(), GemServiceError>;
}
