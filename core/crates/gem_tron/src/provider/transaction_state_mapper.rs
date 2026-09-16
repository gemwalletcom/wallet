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

    #[test]
    fn test_map_transaction_status_confirmed() {
        let receipt = TransactionReceiptData {
            fee: Some(100),
            block_number: 10,
            ..TransactionReceiptData::mock_with_result("SUCCESS")
        };

        let result = map_transaction_status(Some(&receipt));
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(100))]);
    }

    #[test]
    fn test_map_transaction_status_reverted() {
        let revert = TransactionReceiptData {
            result: Some(RECEIPT_FAILED.to_string()),
            fee: Some(854700),
            block_number: 10,
            ..TransactionReceiptData::mock_with_result(RECEIPT_REVERT)
        };
        let result = map_transaction_status(Some(&revert));
        assert_eq!(result.state, TransactionState::Reverted);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(854700))]);

        for (result, receipt_result) in [
            (Some(RECEIPT_FAILED), None),
            (None, Some(RECEIPT_REVERT)),
            (None, Some(RECEIPT_OUT_OF_ENERGY)),
            (None, Some(RECEIPT_FAILED)),
        ] {
            let receipt = TransactionReceiptData {
                result: result.map(str::to_string),
                receipt: TransactionReceipt {
                    result: receipt_result.map(str::to_string),
                },
                fee: Some(100),
                block_number: 10,
                ..TransactionReceiptData::mock_with_result("SUCCESS")
            };
            assert_eq!(map_transaction_status(Some(&receipt)).state, TransactionState::Reverted);
        }
    }

    #[test]
    fn test_map_transaction_status_pending() {
        let receipt = TransactionReceiptData {
            fee: None,
            block_number: 0,
            receipt: TransactionReceipt { result: None },
            ..TransactionReceiptData::mock_with_result("SUCCESS")
        };

        let result = map_transaction_status(Some(&receipt));
        assert_eq!(result.state, TransactionState::Pending);

        assert_eq!(map_transaction_status(None).state, TransactionState::Pending);
    }
}
