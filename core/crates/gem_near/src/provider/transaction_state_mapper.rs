use crate::models::transaction::BroadcastResult;
use num_bigint::{BigInt, BigUint};
use primitives::{TransactionChange, TransactionUpdate};

pub fn map_transaction_status(response: &BroadcastResult) -> TransactionUpdate {
    let changes = vec![TransactionChange::NetworkFee(BigInt::from(
        &response.transaction_outcome.outcome.tokens_burnt * BigUint::from(2u64),
    ))];

    TransactionUpdate::new(response.state(), changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{JsonRpcResult, TransactionState};

    fn result(json: &str) -> BroadcastResult {
        serde_json::from_str::<JsonRpcResult<BroadcastResult>>(json).unwrap().result
    }

    #[test]
    fn test_map_transaction_status() {
        let confirmed = map_transaction_status(&result(include_str!("../../testdata/transaction_transfer_success.json")));
        assert_eq!(confirmed.state, TransactionState::Confirmed);
        assert_eq!(confirmed.changes, vec![TransactionChange::NetworkFee("834989537500000000000".parse::<BigInt>().unwrap())]);

        let failed = map_transaction_status(&result(include_str!("../../testdata/transaction_transfer_failure.json")));
        assert_eq!(failed.state, TransactionState::Failed);

        let pending = map_transaction_status(&result(include_str!("../../testdata/transaction_transfer_pending.json")));
        assert_eq!(pending.state, TransactionState::Pending);
    }
}
