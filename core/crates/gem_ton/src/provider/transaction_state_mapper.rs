use std::error::Error;

use primitives::{TransactionChange, TransactionStateRequest, TransactionUpdate};

use crate::models::TraceResponse;
use crate::provider::transactions_mapper::{base64_hash_to_hex, map_transaction_state};

pub fn map_transaction_status(request: TransactionStateRequest, traces: TraceResponse) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
    let transaction = traces.root_transaction().ok_or("Transaction not found")?;
    let state = if traces.has_actions() {
        traces.action_state().ok_or("Trace not found")?
    } else {
        map_transaction_state(transaction)
    };

    let mut changes = vec![TransactionChange::NetworkFee(transaction.total_fees.clone().into())];
    if let Some(transaction_hash) = base64_hash_to_hex(&transaction.hash)
        && transaction_hash != request.id
    {
        changes.push(TransactionChange::HashChange {
            old: request.id,
            new: transaction_hash,
        });
    }

    Ok(TransactionUpdate::new(state, changes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MessageTransactions, TraceAction};
    use crate::provider::testkit::{FAILED_SWAP_MESSAGE_HASH, FAILED_SWAP_ROOT_TRANSACTION_HEX_HASH, SUCCESS_SWAP_MESSAGE_HASH};
    use primitives::TransactionState;

    #[test]
    fn test_map_transaction_status_confirmed() {
        let request = TransactionStateRequest::mock_with_id("hash");
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_state_success.json")).unwrap();
        let traces = TraceResponse::mock(transactions.transactions.first().unwrap().clone(), false, vec![]);

        let update = map_transaction_status(request, traces).unwrap();
        assert_eq!(update.state, TransactionState::Confirmed);
        assert!(!update.changes.is_empty());
    }

    #[test]
    fn test_ton_transaction_jetton_transfer_reverted() {
        let request = TransactionStateRequest::mock_with_id("hash");
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_jetton_error_2.json")).unwrap();
        let traces = TraceResponse::mock(transactions.transactions.first().unwrap().clone(), false, vec![]);

        let update = map_transaction_status(request, traces).unwrap();
        assert_eq!(update.state, TransactionState::Reverted);
        assert!(!update.changes.is_empty());
    }

    #[test]
    fn test_map_transaction_status_success_trace_action() {
        let request = TransactionStateRequest::mock_with_id(SUCCESS_SWAP_MESSAGE_HASH);
        let traces = TraceResponse::mock_block_trace(0);

        let update = map_transaction_status(request, traces).unwrap();
        assert_eq!(update.state, TransactionState::Confirmed);
        assert!(!update.changes.is_empty());
        assert!(!update.changes.iter().any(|c| matches!(c, TransactionChange::HashChange { .. })));
    }

    #[test]
    fn test_map_transaction_status_failed_trace_action() {
        let request = TransactionStateRequest::mock_with_id(FAILED_SWAP_MESSAGE_HASH);
        let traces = TraceResponse::mock_block_trace(1);
        let transaction = traces.root_transaction().unwrap().clone();

        let root_update = map_transaction_status(request.clone(), TraceResponse::mock(transaction, false, vec![])).unwrap();
        assert_eq!(root_update.state, TransactionState::Confirmed);

        let update = map_transaction_status(request, traces).unwrap();
        assert_eq!(update.state, TransactionState::Reverted);
        assert!(!update.changes.is_empty());

        let hash_change = update.changes.iter().find_map(|change| match change {
            TransactionChange::HashChange { old, new } => Some((old.as_str(), new.as_str())),
            _ => None,
        });
        assert_eq!(hash_change, Some((FAILED_SWAP_MESSAGE_HASH, FAILED_SWAP_ROOT_TRANSACTION_HEX_HASH)));
    }

    #[test]
    fn test_map_transaction_status_incomplete_trace() {
        let request = TransactionStateRequest::mock_with_id("hash");
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_state_success.json")).unwrap();
        let traces = TraceResponse::mock(
            transactions.transactions.first().unwrap().clone(),
            true,
            vec![TraceAction {
                success: Some(true),
                action_type: None,
                details: None,
            }],
        );

        let update = map_transaction_status(request, traces).unwrap();
        assert_eq!(update.state, TransactionState::Pending);
        assert!(!update.changes.is_empty());
    }
}
