use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionUpdate};

use crate::models::rpc::TransactionStatus;

pub fn map_transaction_status(status: &TransactionStatus) -> TransactionUpdate {
    let changes = vec![TransactionChange::NetworkFee(BigInt::from(status.fee.clone()))];

    TransactionUpdate::new(status.state(), changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_jsonrpc::types::JsonRpcResult;
    use primitives::TransactionState;

    fn status(json: &str) -> TransactionStatus {
        serde_json::from_str::<JsonRpcResult<TransactionStatus>>(json).unwrap().take().unwrap()
    }

    #[test]
    fn test_map_transaction_status() {
        let confirmed = map_transaction_status(&status(include_str!("../testdata/transaction_by_hash.json")));
        assert_eq!(confirmed.state, TransactionState::Confirmed);
        assert_eq!(confirmed.changes, vec![TransactionChange::NetworkFee(BigInt::from(11u32))]);

        let failed = map_transaction_status(&status(include_str!("../testdata/transaction_status_failed.json")));
        assert_eq!(failed.state, TransactionState::Failed);

        let pending = map_transaction_status(&status(include_str!("../testdata/transaction_status_pending.json")));
        assert_eq!(pending.state, TransactionState::Pending);
    }
}
