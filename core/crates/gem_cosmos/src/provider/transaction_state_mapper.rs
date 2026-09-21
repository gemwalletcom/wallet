use primitives::{TransactionState, TransactionUpdate};

use crate::models::TransactionResponse;

pub fn map_transaction_status(transaction: TransactionResponse) -> TransactionUpdate {
    let state = if transaction.tx_response.code == 0 { TransactionState::Confirmed } else { TransactionState::Reverted };

    TransactionUpdate::new_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_map_transaction_status_confirmed() {
        let update = map_transaction_status(TransactionResponse::mock_delegate());
        assert_eq!(update.state, TransactionState::Confirmed);
    }

    #[test]
    fn test_map_transaction_status_reverted() {
        let update = map_transaction_status(TransactionResponse::mock_reverted_transfer_spam());
        assert_eq!(update.state, TransactionState::Reverted);
    }
}
