use crate::services::error::GemServiceError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use primitives::{InAppNotification, WalletId};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemNotificationStore: Send + Sync {
    async fn save_notifications(&self, notifications: Vec<InAppNotification>) -> Result<(), GemServiceError>;
    async fn mark_notifications_read(&self, wallet_id: WalletId, created_before: DateTime<Utc>) -> Result<(), GemServiceError>;
    async fn has_unread_notifications(&self, wallet_id: WalletId) -> Result<bool, GemServiceError>;
}
