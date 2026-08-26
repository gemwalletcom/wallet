use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::WalletId;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemWalletConfigurationStore: Send + Sync {
    async fn is_completed(&self, wallet_id: WalletId) -> Result<bool, GemServiceError>;
    async fn set_completed(&self, wallet_id: WalletId) -> Result<(), GemServiceError>;
}
