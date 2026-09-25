use std::sync::{Arc, Mutex};

use primitives::{Transaction, TransactionId, TransactionState, WalletId};

use super::tracker::GemTransactionUpdater;
use super::{GemPendingTransaction, GemTransactionStateResult, GemTransactionStateStore, GemTransactionStateUpdate, GemTransactionStatusService};
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct RecordingTransactionStatus {
    pub tracked: Mutex<Vec<Vec<Transaction>>>,
}

impl GemTransactionStatusService for RecordingTransactionStatus {
    fn track(&self, _: WalletId, transactions: Vec<Transaction>) {
        self.tracked.lock().unwrap().push(transactions);
    }
}

#[derive(Default)]
pub struct MemoryTransactionStateStore {
    pub pending: Mutex<Vec<GemPendingTransaction>>,
    pub states: Mutex<Vec<(TransactionId, TransactionState)>>,
    pub updates: Mutex<Vec<(TransactionId, GemTransactionStateUpdate)>>,
    pub hash_updates: Mutex<Vec<(TransactionId, TransactionId)>>,
    pub deleted: Mutex<Vec<TransactionId>>,
    pub added: Mutex<Vec<(WalletId, Vec<Transaction>)>>,
    pub add_failures: Mutex<usize>,
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
    async fn get_pending_transactions(&self, _states: Vec<TransactionState>) -> Result<Vec<GemPendingTransaction>, GemServiceError> {
        Ok(self.pending.lock().unwrap().clone())
    }

    async fn get_transaction(&self, _wallet_id: WalletId, transaction_id: TransactionId) -> Result<Option<GemPendingTransaction>, GemServiceError> {
        Ok(self.pending.lock().unwrap().iter().find(|pending| pending.transaction.id == transaction_id).cloned())
    }

    async fn add_transactions(&self, wallet_id: WalletId, transactions: Vec<Transaction>) -> Result<(), GemServiceError> {
        let mut failures = self.add_failures.lock().unwrap();
        if *failures > 0 {
            *failures -= 1;
            return Err(GemServiceError::Platform { msg: "database is locked".into() });
        }
        self.added.lock().unwrap().push((wallet_id, transactions));
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

#[derive(Default)]
pub struct TestTransactionUpdater {
    pub results: Mutex<Vec<Result<Option<GemTransactionStateResult>, GemServiceError>>>,
    pub requested: Mutex<Vec<TransactionId>>,
}

#[async_trait::async_trait]
impl GemTransactionUpdater for TestTransactionUpdater {
    async fn update(&self, _wallet_id: WalletId, transaction: Transaction) -> Result<Option<GemTransactionStateResult>, GemServiceError> {
        self.requested.lock().unwrap().push(transaction.id.clone());
        let mut results = self.results.lock().unwrap();
        if results.is_empty() {
            return Ok(None);
        }
        results.remove(0)
    }
}

impl GemTransactionStateResult {
    pub fn mock(transaction_id: TransactionId, state: TransactionState) -> Self {
        Self { transaction_id, state, failures: Vec::new() }
    }
}
