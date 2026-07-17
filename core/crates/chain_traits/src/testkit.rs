use primitives::{Transaction, TransactionId};

use crate::TransactionsResult;

impl TransactionsResult {
    pub fn transactions(&self) -> Option<&[Transaction]> {
        match self {
            Self::Transactions(transactions) => Some(transactions.as_slice()),
            Self::TransactionIds(_) => None,
        }
    }

    pub fn transaction_ids(&self) -> Option<&[TransactionId]> {
        match self {
            Self::Transactions(_) => None,
            Self::TransactionIds(transaction_ids) => Some(transaction_ids.as_slice()),
        }
    }
}
