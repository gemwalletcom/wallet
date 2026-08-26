use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::WalletId;

use super::model::GemDiscoveryStep;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemAssetDiscoveryStore: Send + Sync {
    async fn get_assets_timestamp(&self, wallet_id: WalletId) -> Result<u64, GemServiceError>;
    async fn set_assets_timestamp(&self, wallet_id: WalletId, timestamp: u64) -> Result<(), GemServiceError>;
    async fn is_completed(&self, wallet_id: WalletId, step: GemDiscoveryStep) -> Result<bool, GemServiceError>;
    async fn set_completed(&self, wallet_id: WalletId, step: GemDiscoveryStep) -> Result<(), GemServiceError>;
}
