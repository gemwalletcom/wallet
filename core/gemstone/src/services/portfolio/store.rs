use crate::services::balance::model::GemAssetBalance;
use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::WalletId;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemPortfolioStore: Send + Sync {
    async fn get_wallet_balances(&self, wallet_id: WalletId) -> Result<Vec<GemAssetBalance>, GemServiceError>;
}
