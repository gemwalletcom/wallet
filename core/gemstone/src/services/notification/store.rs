use async_trait::async_trait;
use primitives::{InAppNotification, WalletId};

use super::error::GemNotificationError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemNotificationStore: Send + Sync {
    async fn save(&self, notifications: Vec<InAppNotification>) -> Result<(), GemNotificationError>;
    async fn get_sync_timestamp(&self, wallet_id: WalletId) -> Result<u64, GemNotificationError>;
    async fn set_sync_timestamp(&self, wallet_id: WalletId, timestamp: u64) -> Result<(), GemNotificationError>;
}
