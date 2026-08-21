use num_bigint::BigUint;
use primitives::TransactionState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    SuccessReceiptId(String),
    SuccessValue(String),
    Failure(Value),
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
    #[serde(rename = "FunctionCall")]
    pub function_call: Option<FunctionCallAction>,
    #[serde(rename = "Delegate")]
    pub delegate: Option<SignedDelegateAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferAction {
    pub deposit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallAction {
    pub deposit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedDelegateAction {
    pub delegate_action: DelegateAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateAction {
    pub sender_id: String,
    pub receiver_id: String,
    pub actions: Vec<TransactionAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutcome {
    pub outcome: Outcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub logs: Vec<String>,
    pub status: ExecutionStatus,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub tokens_burnt: BigUint,
}
