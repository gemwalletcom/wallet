use async_trait::async_trait;
use primitives::{AssetId, Transaction};

use super::error::GemTransactionsError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemTransactionStore: Send + Sync {
    async fn get_sync_timestamp(&self, wallet_id: String, asset_id: Option<AssetId>) -> Result<u64, GemTransactionsError>;
    async fn set_sync_timestamp(&self, wallet_id: String, asset_id: Option<AssetId>, timestamp: u64) -> Result<(), GemTransactionsError>;
    async fn add_transactions(&self, wallet_id: String, transactions: Vec<Transaction>) -> Result<(), GemTransactionsError>;
}
