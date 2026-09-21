use crate::rpc::model::TransactionReceipt;
use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};

pub fn map_transaction_status(receipt: &TransactionReceipt) -> TransactionUpdate {
    map_transaction_status_with_fee(receipt, receipt.get_fee().into())
}

pub fn map_transaction_status_with_fee(receipt: &TransactionReceipt, network_fee: BigInt) -> TransactionUpdate {
    let state = match receipt.get_state() {
        TransactionState::Confirmed => TransactionState::Confirmed,
        TransactionState::Reverted => TransactionState::Reverted,
        TransactionState::Pending | TransactionState::InTransit | TransactionState::Failed | TransactionState::Refunded => {
            return TransactionUpdate::new_state(TransactionState::Pending);
        }
    };
    TransactionUpdate::new(state, vec![TransactionChange::BlockNumber(receipt.block_number.to_string()), TransactionChange::NetworkFee(network_fee)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::{BigInt, BigUint};

    #[test]
    fn test_map_transaction_status() {
        let receipt = TransactionReceipt {
            block_number: 0x123,
            ..TransactionReceipt::mock()
        };

        let result = map_transaction_status(&receipt);

        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::BlockNumber("291".to_string()), TransactionChange::NetworkFee(BigInt::from(420000000000000u64))]);

        let result = map_transaction_status(&TransactionReceipt {
            status: "0x0".to_string(),
            ..receipt.clone()
        });

        assert_eq!(result.state, TransactionState::Reverted);
        assert_eq!(result.changes, vec![TransactionChange::BlockNumber("291".to_string()), TransactionChange::NetworkFee(BigInt::from(420000000000000u64))]);

        let result = map_transaction_status(&TransactionReceipt {
            status: "0x2".to_string(),
            ..receipt.clone()
        });

        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes, vec![]);

        let result = map_transaction_status(&TransactionReceipt {
            block_hash: primitives::contract_constants::EVM_ZERO_BLOCK_HASH.to_string(),
            ..receipt.clone()
        });

        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes, vec![]);

        let result = map_transaction_status(&TransactionReceipt { block_number: 0, ..receipt.clone() });

        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes, vec![]);

        let result = map_transaction_status(&TransactionReceipt {
            l1_fee: Some(BigUint::from(5000000000000000u64)),
            ..receipt
        });

        assert_eq!(result.state, TransactionState::Confirmed);
        let expected_total = BigInt::from(21000u32) * BigInt::from(20000000000u64) + BigInt::from(5000000000000000u64);
        assert_eq!(result.changes, vec![TransactionChange::BlockNumber("291".to_string()), TransactionChange::NetworkFee(expected_total)]);
    }
}
