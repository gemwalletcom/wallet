use crate::constants::{TRANSACTION_STATUS_EXECUTED, TRANSACTION_STATUS_EXECUTED_OPTIMISTIC, TRANSACTION_STATUS_FINAL};
use crate::models::transaction::BroadcastResult;
use std::error::Error;

pub fn map_transaction_broadcast(response: &BroadcastResult) -> Result<String, Box<dyn Error + Sync + Send>> {
    match response.final_execution_status.as_str() {
        TRANSACTION_STATUS_FINAL | TRANSACTION_STATUS_EXECUTED | TRANSACTION_STATUS_EXECUTED_OPTIMISTIC => Ok(response.transaction.hash.clone()),
        _ => Err(format!("Broadcast failed with status: {}", response.final_execution_status).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{BroadcastResult, BroadcastTransaction, Outcome, TransactionOutcome};
    use primitives::JsonRpcResult;

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
                tokens_burnt: tokens_burnt.parse().unwrap(),
            },
        }
    }

    #[test]
    fn test_map_transaction_broadcast_success() {
        for status in [TRANSACTION_STATUS_FINAL, TRANSACTION_STATUS_EXECUTED, TRANSACTION_STATUS_EXECUTED_OPTIMISTIC] {
            let response = BroadcastResult {
                final_execution_status: status.to_string(),
                transaction: create_test_transaction(),
                transaction_outcome: create_test_outcome("417494768750000000000"),
            };

            let result = map_transaction_broadcast(&response).unwrap();
            assert_eq!(result, "5qSP5dRVr5KQ37Dd9CV2gi7KDuvtU4eFaRK7cDKREVL2");
        }
    }

    #[test]
    fn test_map_transaction_broadcast_failure() {
        let response = BroadcastResult {
            final_execution_status: "EXECUTION_FAILURE".to_string(),
            transaction: create_test_transaction(),
            transaction_outcome: create_test_outcome("0"),
        };

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("EXECUTION_FAILURE"));
    }

    #[test]
    fn test_map_real_transaction_response() {
        let response: JsonRpcResult<BroadcastResult> = serde_json::from_str(include_str!("../../testdata/successful_transaction.json")).unwrap();

        let hash = map_transaction_broadcast(&response.result).unwrap();
        assert_eq!(hash, "5qSP5dRVr5KQ37Dd9CV2gi7KDuvtU4eFaRK7cDKREVL2");
    }
}
