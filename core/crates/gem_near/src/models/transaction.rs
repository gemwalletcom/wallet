use num_bigint::BigUint;
use primitives::TransactionState;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

use crate::constants::TRANSACTION_STATUSES_EXECUTED;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResult {
    pub final_execution_status: String,
    pub status: ExecutionStatus,
    pub transaction: BroadcastTransaction,
    pub transaction_outcome: TransactionOutcome,
}

impl BroadcastResult {
    pub fn state(&self) -> TransactionState {
        match &self.status {
            ExecutionStatus::Failure(_) => TransactionState::Failed,
            ExecutionStatus::SuccessValue(_) if self.is_executed() => TransactionState::Confirmed,
            _ => TransactionState::Pending,
        }
    }

    pub fn is_executed(&self) -> bool {
        TRANSACTION_STATUSES_EXECUTED.contains(&self.final_execution_status.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    NotStarted,
    Started,
    SuccessValue(String),
    Failure(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastTransaction {
    pub hash: String,
    pub signer_id: String,
    pub receiver_id: String,
    pub actions: Vec<TransactionAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAction {
    #[serde(rename = "Transfer")]
    pub transfer: Option<TransferAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferAction {
    pub deposit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutcome {
    pub outcome: Outcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub tokens_burnt: BigUint,
}
