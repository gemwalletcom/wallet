use async_trait::async_trait;
use primitives::InAppNotification;

use super::error::GemNotificationError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemNotificationStore: Send + Sync {
    async fn save(&self, notifications: Vec<InAppNotification>) -> Result<(), GemNotificationError>;
    async fn get_sync_timestamp(&self, wallet_id: String) -> Result<u64, GemNotificationError>;
    async fn set_sync_timestamp(&self, wallet_id: String, timestamp: u64) -> Result<(), GemNotificationError>;
}
