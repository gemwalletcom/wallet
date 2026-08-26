use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{InAppNotification, WalletId};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemNotificationStore: Send + Sync {
    async fn save(&self, notifications: Vec<InAppNotification>) -> Result<(), GemServiceError>;
    async fn get_sync_timestamp(&self, wallet_id: WalletId) -> Result<u64, GemServiceError>;
    async fn set_sync_timestamp(&self, wallet_id: WalletId, timestamp: u64) -> Result<(), GemServiceError>;
}
