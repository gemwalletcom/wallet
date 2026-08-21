use crate::models::transaction::BroadcastResult;
use std::error::Error;

pub fn map_transaction_broadcast(response: &BroadcastResult) -> Result<String, Box<dyn Error + Sync + Send>> {
    if response.is_executed() {
        Ok(response.transaction.hash.clone())
    } else {
        Err(format!("Broadcast failed with status: {}", response.final_execution_status).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::TRANSACTION_STATUSES_EXECUTED;
    use crate::models::transaction::{BroadcastResult, BroadcastTransaction, ExecutionStatus, Outcome, TransactionOutcome};
    use primitives::JsonRpcResult;
    use serde_json::Value;

    fn create_test_transaction() -> BroadcastTransaction {
        BroadcastTransaction {
            hash: "5qSP5dRVr5KQ37Dd9CV2gi7KDuvtU4eFaRK7cDKREVL2".to_string(),
            signer_id: "test.near".to_string(),
            receiver_id: "receiver.near".to_string(),
            actions: vec![],
        }
    }

    fn create_test_outcome(tokens_burnt: &str) -> TransactionOutcome {
        TransactionOutcome {
            outcome: Outcome {
                executor_id: None,
                logs: Vec::new(),
                status: ExecutionStatus::SuccessValue(String::new()),
                tokens_burnt: tokens_burnt.parse().unwrap(),
            },
        }
    }

    #[test]
    fn test_map_transaction_broadcast_success() {
        for status in TRANSACTION_STATUSES_EXECUTED {
            let response = BroadcastResult {
                final_execution_status: status.to_string(),
                status: ExecutionStatus::SuccessValue(String::new()),
                transaction: create_test_transaction(),
                transaction_outcome: create_test_outcome("417494768750000000000"),
                receipts_outcome: vec![],
            };

            let result = map_transaction_broadcast(&response).unwrap();
            assert_eq!(result, "5qSP5dRVr5KQ37Dd9CV2gi7KDuvtU4eFaRK7cDKREVL2");
        }
    }

    #[test]
    fn test_map_transaction_broadcast_failure() {
        let response = BroadcastResult {
            final_execution_status: "EXECUTION_FAILURE".to_string(),
            status: ExecutionStatus::Failure(Value::Null),
            transaction: create_test_transaction(),
            transaction_outcome: create_test_outcome("0"),
            receipts_outcome: vec![],
        };

        let error = map_transaction_broadcast(&response).unwrap_err();
        assert_eq!(error.to_string(), "Broadcast failed with status: EXECUTION_FAILURE");
    }

    #[test]
    fn test_map_real_transaction_response() {
        let response: JsonRpcResult<BroadcastResult> = serde_json::from_str(include_str!("../../testdata/successful_transaction.json")).unwrap();

        let hash = map_transaction_broadcast(&response.result).unwrap();
        assert_eq!(hash, "5qSP5dRVr5KQ37Dd9CV2gi7KDuvtU4eFaRK7cDKREVL2");
    }
}
