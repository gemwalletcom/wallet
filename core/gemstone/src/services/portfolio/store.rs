use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{AssetData, WalletId};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemPortfolioStore: Send + Sync {
    async fn get_portfolio_assets(&self, wallet_id: WalletId) -> Result<Vec<AssetData>, GemServiceError>;
}
