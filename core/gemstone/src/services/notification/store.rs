use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::InAppNotification;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemNotificationStore: Send + Sync {
    async fn save_notifications(&self, notifications: Vec<InAppNotification>) -> Result<(), GemServiceError>;
}
