use crate::services::error::GemServiceError;
use async_trait::async_trait;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemNotificationPermissions: Send + Sync {
    async fn request_permissions_or_open_settings(&self) -> Result<bool, GemServiceError>;
}
