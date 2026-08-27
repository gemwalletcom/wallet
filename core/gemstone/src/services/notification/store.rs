use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::InAppNotification;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemNotificationStore: Send + Sync {
    async fn save(&self, notifications: Vec<InAppNotification>) -> Result<(), GemServiceError>;
}
