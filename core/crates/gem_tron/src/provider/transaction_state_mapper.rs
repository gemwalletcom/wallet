use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};

use crate::models::TransactionReceiptData;

pub fn map_transaction_status(receipt: Option<&TransactionReceiptData>) -> TransactionUpdate {
    let Some(receipt) = receipt else {
        return TransactionUpdate::new_state(TransactionState::Pending);
    };

    let changes = receipt.fee.map(|fee| vec![TransactionChange::NetworkFee(BigInt::from(fee))]).unwrap_or_default();

    if receipt.is_failed() {
        return TransactionUpdate::new(TransactionState::Reverted, changes);
    }

    if receipt.block_number > 0 {
        return TransactionUpdate::new(TransactionState::Confirmed, changes);
    }

    TransactionUpdate::new_state(TransactionState::Pending)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RECEIPT_FAILED, RECEIPT_OUT_OF_ENERGY, RECEIPT_REVERT, TransactionReceipt, TransactionReceiptData};

    fn create_receipt(result: Option<&str>, receipt_result: Option<&str>, block_number: i64, fee: Option<u64>) -> TransactionReceiptData {
        TransactionReceiptData {
            id: "transaction_id".to_string(),
            fee,
            block_number,
            block_time_stamp: 0,
            result: result.map(|value| value.to_string()),
            receipt: TransactionReceipt {
                result: receipt_result.map(|value| value.to_string()),
            },
            log: None,
            internal_transactions: None,
        }
    }

    #[test]
    fn test_map_transaction_status_confirmed() {
        let receipt = create_receipt(None, Some("SUCCESS"), 10, Some(100));

        let result = map_transaction_status(Some(&receipt));
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(100))]);
    }

    #[test]
    fn test_map_transaction_status_reverted() {
        let revert = create_receipt(Some(RECEIPT_FAILED), Some(RECEIPT_REVERT), 10, Some(854700));
        let result = map_transaction_status(Some(&revert));
        assert_eq!(result.state, TransactionState::Reverted);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(854700))]);

        let top_level_failed_only = create_receipt(Some(RECEIPT_FAILED), None, 10, Some(100));
        assert_eq!(map_transaction_status(Some(&top_level_failed_only)).state, TransactionState::Reverted);

        let receipt_revert_only = create_receipt(None, Some(RECEIPT_REVERT), 10, Some(100));
        assert_eq!(map_transaction_status(Some(&receipt_revert_only)).state, TransactionState::Reverted);

        let out_of_energy = create_receipt(None, Some(RECEIPT_OUT_OF_ENERGY), 10, Some(100));
        assert_eq!(map_transaction_status(Some(&out_of_energy)).state, TransactionState::Reverted);

        let receipt_failed = create_receipt(None, Some(RECEIPT_FAILED), 10, Some(100));
        assert_eq!(map_transaction_status(Some(&receipt_failed)).state, TransactionState::Reverted);
    }

    #[test]
    fn test_map_transaction_status_pending() {
        let receipt = create_receipt(None, None, 0, None);

        let result = map_transaction_status(Some(&receipt));
        assert_eq!(result.state, TransactionState::Pending);

        assert_eq!(map_transaction_status(None).state, TransactionState::Pending);
    }
}
