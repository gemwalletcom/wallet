use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{PortfolioAsset, WalletId};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemPortfolioStore: Send + Sync {
    async fn get_wallet_assets(&self, wallet_id: WalletId) -> Result<Vec<PortfolioAsset>, GemServiceError>;
}
