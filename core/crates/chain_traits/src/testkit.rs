use primitives::{Transaction, TransactionIdRequest};

use crate::TransactionsResult;

impl TransactionsResult {
    pub fn transactions(&self) -> Option<&[Transaction]> {
        match self {
            Self::Transactions(transactions) => Some(transactions.as_slice()),
            Self::TransactionRequests(_) => None,
        }
    }

    pub fn transaction_requests(&self) -> Option<&[TransactionIdRequest]> {
        match self {
            Self::Transactions(_) => None,
            Self::TransactionRequests(transaction_requests) => Some(transaction_requests.as_slice()),
        }
    }
}
