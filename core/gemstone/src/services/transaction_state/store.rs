use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{Transaction, TransactionId, TransactionState, WalletId};

use super::model::{GemPendingTransaction, GemTransactionStateUpdate};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemTransactionStateStore: Send + Sync {
    async fn get_pending_transactions(&self) -> Result<Vec<GemPendingTransaction>, GemServiceError>;
    async fn get_transaction(&self, wallet_id: WalletId, transaction_id: TransactionId) -> Result<Option<GemPendingTransaction>, GemServiceError>;
    async fn add_transactions(&self, wallet_id: WalletId, transactions: Vec<Transaction>) -> Result<(), GemServiceError>;
    async fn get_state(&self, wallet_id: WalletId, transaction_id: TransactionId) -> Result<Option<TransactionState>, GemServiceError>;
    async fn update_transaction_hash(&self, wallet_id: WalletId, transaction_id: TransactionId, hash: String) -> Result<(), GemServiceError>;
    async fn delete_transaction(&self, wallet_id: WalletId, transaction_id: TransactionId) -> Result<(), GemServiceError>;
    async fn update_transaction(&self, wallet_id: WalletId, transaction_id: TransactionId, update: GemTransactionStateUpdate) -> Result<bool, GemServiceError>;
}
