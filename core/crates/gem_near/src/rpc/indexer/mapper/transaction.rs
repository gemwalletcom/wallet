use std::error::Error;

use primitives::Transaction;

use super::super::model::FastNearTransaction;
use crate::rpc::mapper::{ReceiptOutcome, map_transaction};

pub(in crate::rpc::indexer) fn map_raw_transaction(transaction: FastNearTransaction) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
    let state = transaction.state();
    let fee = transaction.fee();
    let block_height = transaction.execution_outcome.block_height;
    let block_timestamp = transaction.execution_outcome.block_timestamp;
    let receipts = transaction
        .receipts
        .into_iter()
        .map(|receipt| ReceiptOutcome {
            receiver_id: receipt.receipt.receiver_id,
            outcome: receipt.execution_outcome.outcome,
        })
        .collect();
    map_transaction(transaction.transaction, receipts, block_height, block_timestamp, state, fee)
}
