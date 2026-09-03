use primitives::{Transaction, WalletId};

#[uniffi::export(with_foreign)]
pub trait GemTransactionTracking: Send + Sync {
    fn track(&self, wallet_id: WalletId, transactions: Vec<Transaction>);
}
