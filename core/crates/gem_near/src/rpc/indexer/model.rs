use std::iter;

use num_bigint::{BigInt, BigUint};
use primitives::TransactionState;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_bigint_from_str, deserialize_u64_from_str};

use crate::models::transaction::{BroadcastTransaction, ExecutionStatus, Outcome};

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum TransferDirection {
    Sender,
    Receiver,
}

#[derive(Debug, Serialize)]
pub(super) struct TransfersRequest<'a> {
    pub account_id: &'a str,
    pub direction: TransferDirection,
    pub desc: bool,
    pub limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_timestamp_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(super) struct TransfersResponse {
    pub transfers: Vec<FastNearTransfer>,
}

#[derive(Debug, Deserialize)]
pub(super) struct FastNearTransfer {
    pub account_id: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub amount: BigInt,
    pub asset_id: String,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub block_timestamp: u64,
    pub predecessor_id: String,
    pub other_account_id: Option<String>,
    pub receipt_account_id: String,
    pub receipt_id: String,
    pub signer_id: String,
    pub transaction_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct TransactionsRequest<'a> {
    pub tx_hashes: &'a [String],
}

#[derive(Debug, Deserialize)]
pub(super) struct TransactionsResponse {
    pub transactions: Vec<FastNearTransaction>,
}

#[derive(Debug, Deserialize)]
pub(super) struct NearDataBlockResponse {
    pub block: NearDataBlock,
    pub shards: Vec<NearDataShard>,
}

#[derive(Debug, Deserialize)]
pub(super) struct NearDataBlock {
    pub header: NearDataBlockHeader,
}

#[derive(Debug, Deserialize)]
pub(super) struct NearDataBlockHeader {
    pub height: u64,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize)]
pub(super) struct NearDataShard {
    pub chunk: Option<NearDataChunk>,
}

#[derive(Debug, Deserialize)]
pub(super) struct NearDataChunk {
    pub transactions: Vec<NearDataTransaction>,
}

#[derive(Debug, Deserialize)]
pub(super) struct NearDataTransaction {
    pub outcome: NearDataTransactionOutcome,
    pub transaction: BroadcastTransaction,
}

#[derive(Debug, Deserialize)]
pub(super) struct NearDataTransactionOutcome {
    pub execution_outcome: crate::models::TransactionOutcome,
}

#[derive(Debug, Deserialize)]
pub(super) struct FastNearTransaction {
    pub execution_outcome: FastNearExecutionOutcome,
    pub receipts: Vec<FastNearReceipt>,
    pub transaction: BroadcastTransaction,
}

#[derive(Debug, Deserialize)]
pub(super) struct FastNearReceipt {
    pub execution_outcome: FastNearExecutionOutcome,
    pub receipt: FastNearReceiptData,
}

#[derive(Debug, Deserialize)]
pub(super) struct FastNearReceiptData {
    pub receiver_id: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct FastNearExecutionOutcome {
    pub block_height: u64,
    pub block_timestamp: u64,
    pub outcome: Outcome,
}

impl FastNearTransaction {
    pub(super) fn fee(&self) -> BigUint {
        self.receipts.iter().fold(self.execution_outcome.outcome.tokens_burnt.clone(), |fee, receipt| {
            fee + &receipt.execution_outcome.outcome.tokens_burnt
        })
    }

    pub(super) fn state(&self) -> TransactionState {
        let mut pending = false;
        for outcome in iter::once(&self.execution_outcome).chain(self.receipts.iter().map(|receipt| &receipt.execution_outcome)) {
            match &outcome.outcome.status {
                ExecutionStatus::Failure(_) => return TransactionState::Failed,
                ExecutionStatus::NotStarted | ExecutionStatus::Started => pending = true,
                ExecutionStatus::SuccessReceiptId(_) | ExecutionStatus::SuccessValue(_) => {}
            }
        }
        if pending { TransactionState::Pending } else { TransactionState::Confirmed }
    }
}
