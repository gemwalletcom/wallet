use std::sync::{Arc, Mutex};

use primitives::{Transaction, TransactionId, TransactionState, WalletId};

use super::{GemPendingTransaction, GemTransactionStateStore, GemTransactionStateUpdate};
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryTransactionStateStore {
    pub states: Mutex<Vec<(TransactionId, TransactionState)>>,
    pub updates: Mutex<Vec<(TransactionId, GemTransactionStateUpdate)>>,
    pub hash_updates: Mutex<Vec<(TransactionId, TransactionId)>>,
    pub deleted: Mutex<Vec<TransactionId>>,
}

impl MemoryTransactionStateStore {
    pub fn with(states: Vec<(TransactionId, TransactionState)>) -> Arc<Self> {
        Arc::new(Self {
            states: Mutex::new(states),
            ..Default::default()
        })
    }
}

#[async_trait::async_trait]
impl GemTransactionStateStore for MemoryTransactionStateStore {
    async fn get_pending_transactions(&self) -> Result<Vec<GemPendingTransaction>, GemServiceError> {
        Ok(Vec::new())
    }

    async fn get_transaction(&self, _wallet_id: WalletId, _transaction_id: TransactionId) -> Result<Option<GemPendingTransaction>, GemServiceError> {
        Ok(None)
    }

    async fn add_transactions(&self, _wallet_id: WalletId, _transactions: Vec<Transaction>) -> Result<(), GemServiceError> {
        Ok(())
    }

    async fn get_state(&self, _wallet_id: WalletId, transaction_id: TransactionId) -> Result<Option<TransactionState>, GemServiceError> {
        Ok(self.states.lock().unwrap().iter().find(|(id, _)| *id == transaction_id).map(|(_, state)| *state))
    }
    async fn update_transaction_hash(&self, _wallet_id: WalletId, transaction_id: TransactionId, hash: String) -> Result<(), GemServiceError> {
        let new_transaction_id = TransactionId::new(transaction_id.chain, hash);
        if transaction_id == new_transaction_id {
            return Ok(());
        }
        let mut states = self.states.lock().unwrap();
        let Some(index) = states.iter().position(|(id, _)| *id == transaction_id) else {
            return Ok(());
        };
        let (_, state) = states.remove(index);
        if !states.iter().any(|(id, _)| *id == new_transaction_id) {
            states.push((new_transaction_id.clone(), state));
        }
        self.hash_updates.lock().unwrap().push((transaction_id, new_transaction_id));
        Ok(())
    }
    async fn delete_transaction(&self, _wallet_id: WalletId, transaction_id: TransactionId) -> Result<(), GemServiceError> {
        self.states.lock().unwrap().retain(|(id, _)| *id != transaction_id);
        self.deleted.lock().unwrap().push(transaction_id);
        Ok(())
    }
    async fn update_transaction(&self, _wallet_id: WalletId, transaction_id: TransactionId, update: GemTransactionStateUpdate) -> Result<bool, GemServiceError> {
        let mut states = self.states.lock().unwrap();
        let Some(entry) = states.iter_mut().find(|(id, _)| *id == transaction_id) else {
            return Ok(false);
        };
        entry.1 = update.state;
        self.updates.lock().unwrap().push((transaction_id, update));
        Ok(true)
    }
}
