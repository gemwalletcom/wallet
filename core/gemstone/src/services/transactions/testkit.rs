use std::sync::Mutex;

use async_trait::async_trait;
use primitives::{Transaction, WalletId};

use super::store::GemTransactionStore;
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryTransactionStore {
    pub saved: Mutex<Vec<Vec<Transaction>>>,
}

#[async_trait]
impl GemTransactionStore for MemoryTransactionStore {
    async fn save_transactions(&self, _: WalletId, transactions: Vec<Transaction>) -> Result<(), GemServiceError> {
        self.saved.lock().unwrap().push(transactions);
        Ok(())
    }
}
