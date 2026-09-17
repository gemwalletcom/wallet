use crate::constants::TRANSACTION_STATUS_FINAL;
use crate::models::transaction::{BroadcastResult, BroadcastTransaction, ExecutionStatus, Outcome, TransactionOutcome};

impl BroadcastResult {
    pub fn mock() -> Self {
        Self {
            final_execution_status: TRANSACTION_STATUS_FINAL.to_string(),
            status: ExecutionStatus::SuccessValue(String::new()),
            transaction: BroadcastTransaction {
                hash: "5qSP5dRVr5KQ37Dd9CV2gi7KDuvtU4eFaRK7cDKREVL2".to_string(),
                signer_id: "test.near".to_string(),
                receiver_id: "receiver.near".to_string(),
                actions: vec![],
            },
            transaction_outcome: TransactionOutcome {
                outcome: Outcome {
                    executor_id: None,
                    logs: vec![],
                    status: ExecutionStatus::SuccessValue(String::new()),
                    tokens_burnt: "417494768750000000000".parse().unwrap(),
                },
            },
            receipts_outcome: vec![],
        }
    }
}
