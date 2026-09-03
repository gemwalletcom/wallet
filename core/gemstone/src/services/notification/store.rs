use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{InAppNotification, WalletId};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemNotificationStore: Send + Sync {
    async fn save_notifications(&self, notifications: Vec<InAppNotification>) -> Result<(), GemServiceError>;
    async fn has_unread_notifications(&self, wallet_id: WalletId) -> Result<bool, GemServiceError>;
}
