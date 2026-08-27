use crate::services::error::GemServiceError;
use async_trait::async_trait;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemDeviceSync: Send + Sync {
    async fn sync_device(&self) -> Result<(), GemServiceError>;
}
