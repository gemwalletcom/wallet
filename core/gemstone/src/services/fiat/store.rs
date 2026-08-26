use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{FiatTransactionData, WalletId};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemFiatStore: Send + Sync {
    async fn save_transactions(&self, wallet_id: WalletId, transactions: Vec<FiatTransactionData>) -> Result<(), GemServiceError>;
}
