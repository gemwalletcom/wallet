use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};

use crate::models::rpc::{AccountLedger, TransactionStatus};

pub fn map_transaction_status(status: &TransactionStatus) -> TransactionUpdate {
    transaction_update(status, status.state())
}

pub fn map_batch_transaction_status(status: &TransactionStatus, batch_id: &str, ledger: &AccountLedger) -> TransactionUpdate {
    let state = match ledger.transactions.iter().any(|transaction| transaction.meta.is_batch_part(batch_id)) {
        true => TransactionState::Confirmed,
        false => TransactionState::Failed,
    };
    transaction_update(status, state)
}

fn transaction_update(status: &TransactionStatus, state: TransactionState) -> TransactionUpdate {
    TransactionUpdate::new(state, vec![TransactionChange::NetworkFee(BigInt::from(status.fee.clone()))])
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

    #[test]
    fn test_map_batch_transaction_status() {
        let batch = status(include_str!("../testdata/transaction_batch.json"));
        let ledger = serde_json::from_str::<JsonRpcResult<AccountLedger>>(include_str!("../testdata/account_transactions_batch_swap.json")).unwrap().take().unwrap();
        let batch_id = "3B925B414BF7E91EC67A3E283F374DE0BC84D6E11DF5345115B44AF7DA5FEDEA";

        assert_eq!(batch.applied_batch_ledger(), Some(107_888_305));
        assert_eq!(status(include_str!("../testdata/transaction_by_hash.json")).applied_batch_ledger(), None);
        assert_eq!(
            map_batch_transaction_status(&batch, &batch_id.to_lowercase(), &ledger),
            TransactionUpdate::new(TransactionState::Confirmed, vec![TransactionChange::NetworkFee(BigInt::from(60u32))])
        );
        assert_eq!(
            map_batch_transaction_status(&batch, "0341D82BA5B421F2E52DC52B33289535C3BE77BC5EB94BA4CA406A93A46380A3", &ledger).state,
            TransactionState::Failed
        );
    }
}
