use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{AssetId, Transaction, WalletId};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemTransactionStore: Send + Sync {
    async fn get_sync_timestamp(&self, wallet_id: WalletId, asset_id: Option<AssetId>) -> Result<u64, GemServiceError>;
    async fn set_sync_timestamp(&self, wallet_id: WalletId, asset_id: Option<AssetId>, timestamp: u64) -> Result<(), GemServiceError>;
    async fn save_transactions(&self, wallet_id: WalletId, transactions: Vec<Transaction>) -> Result<(), GemServiceError>;
}
