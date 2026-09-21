use crate::services::error::GemServiceError;
use async_trait::async_trait;
use std::time::Duration;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemStreamConnection: Send + Sync {
    async fn latency(&self) -> Option<Duration>;
    async fn is_connected(&self) -> bool;
    async fn send(&self, message: String) -> Result<(), GemServiceError>;
}
