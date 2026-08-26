use async_trait::async_trait;
use primitives::WalletId;

use super::error::GemBalanceError;
use super::model::GemBalanceUpdate;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemBalanceStore: Send + Sync {
    async fn update_balances(&self, wallet_id: WalletId, updates: Vec<GemBalanceUpdate>) -> Result<(), GemBalanceError>;
}
