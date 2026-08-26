use async_trait::async_trait;
use primitives::{TransactionId, TransactionState, WalletId};

use super::error::GemTransactionStateError;
use super::model::GemTransactionStateUpdate;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemTransactionStateStore: Send + Sync {
    async fn get_state(&self, wallet_id: WalletId, transaction_id: TransactionId) -> Result<Option<TransactionState>, GemTransactionStateError>;
    async fn rename_transaction(&self, wallet_id: WalletId, transaction_id: TransactionId, new_transaction_id: TransactionId) -> Result<(), GemTransactionStateError>;
    async fn delete_transaction(&self, wallet_id: WalletId, transaction_id: TransactionId) -> Result<(), GemTransactionStateError>;
    async fn update_transaction(&self, wallet_id: WalletId, transaction_id: TransactionId, update: GemTransactionStateUpdate) -> Result<bool, GemTransactionStateError>;
}
