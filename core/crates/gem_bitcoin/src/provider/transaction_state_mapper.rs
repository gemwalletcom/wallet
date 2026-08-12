use primitives::{TransactionChange, TransactionState, TransactionUpdate};

use crate::models::Transaction;

pub fn map_transaction_status(transaction: &Transaction) -> TransactionUpdate {
    if transaction.is_confirmed() {
        return TransactionUpdate::new_state(TransactionState::Confirmed);
    }

    let changes = transaction
        .confirmation_eta_seconds
        .and_then(|seconds| u32::try_from(seconds).ok())
        .filter(|seconds| *seconds > 0)
        .map(TransactionChange::ConfirmationEtaSeconds)
        .into_iter()
        .collect();

    TransactionUpdate::new(TransactionState::Pending, changes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_transaction_status() {
        let confirmed = Transaction::mock();
        assert_eq!(map_transaction_status(&confirmed), TransactionUpdate::new_state(TransactionState::Confirmed));

        let pending_without_estimate = Transaction {
            confirmations: Some(0),
            ..Transaction::mock()
        };
        assert_eq!(map_transaction_status(&pending_without_estimate), TransactionUpdate::new_state(TransactionState::Pending));

        let pending_with_estimate = Transaction {
            confirmations: Some(0),
            confirmation_eta_seconds: Some(720),
            ..Transaction::mock()
        };
        assert_eq!(
            map_transaction_status(&pending_with_estimate),
            TransactionUpdate::new(TransactionState::Pending, vec![TransactionChange::ConfirmationEtaSeconds(720)],),
        );
    }
}
