use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::StreamMessage;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemStreamConnection: Send + Sync {
    async fn is_connected(&self) -> bool;
    async fn send(&self, message: StreamMessage) -> Result<(), GemServiceError>;
}
