use crate::rpc::model::TransactionReciept;
use num_bigint::BigInt;
use primitives::{TransactionChange, TransactionState, TransactionUpdate};

pub fn map_transaction_status(receipt: &TransactionReciept) -> TransactionUpdate {
    let state = match receipt.get_state() {
        TransactionState::Confirmed => TransactionState::Confirmed,
        TransactionState::Reverted => TransactionState::Reverted,
        TransactionState::Pending | TransactionState::InTransit | TransactionState::Failed => return TransactionUpdate::new_state(TransactionState::Pending),
    };
    let network_fee: BigInt = receipt.get_fee().into();
    TransactionUpdate::new(
        state,
        vec![TransactionChange::BlockNumber(receipt.block_number.to_string()), TransactionChange::NetworkFee(network_fee)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    const BLOCK_HASH: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";

    fn receipt(status: &str, block_number: u32, block_hash: &str, l1_fee: Option<BigUint>) -> TransactionReciept {
        TransactionReciept {
            gas_used: BigUint::from(21000u32),
            effective_gas_price: BigUint::from(20000000000u64),
            l1_fee,
            logs: vec![],
            status: status.to_string(),
            block_hash: block_hash.to_string(),
            block_number: BigUint::from(block_number),
        }
    }

    #[test]
    fn test_map_transaction_status() {
        let result = map_transaction_status(&receipt("0x1", 0x123, BLOCK_HASH, None));

        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(
            result.changes,
            vec![
                TransactionChange::BlockNumber("291".to_string()),
                TransactionChange::NetworkFee(BigInt::from(420000000000000u64))
            ]
        );

        let result = map_transaction_status(&receipt("0x0", 0x123, BLOCK_HASH, None));

        assert_eq!(result.state, TransactionState::Reverted);
        assert_eq!(
            result.changes,
            vec![
                TransactionChange::BlockNumber("291".to_string()),
                TransactionChange::NetworkFee(BigInt::from(420000000000000u64))
            ]
        );

        let result = map_transaction_status(&receipt("0x2", 0x123, BLOCK_HASH, None));

        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes, vec![]);

        let result = map_transaction_status(&receipt("0x1", 0x123, primitives::contract_constants::EVM_ZERO_BLOCK_HASH, None));

        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes, vec![]);

        let result = map_transaction_status(&receipt("0x1", 0, BLOCK_HASH, None));

        assert_eq!(result.state, TransactionState::Pending);
        assert_eq!(result.changes, vec![]);

        let result = map_transaction_status(&receipt("0x1", 0x123, BLOCK_HASH, Some(BigUint::from(5000000000000000u64))));

        assert_eq!(result.state, TransactionState::Confirmed);
        let expected_total = BigInt::from(21000u32) * BigInt::from(20000000000u64) + BigInt::from(5000000000000000u64);
        assert_eq!(
            result.changes,
            vec![TransactionChange::BlockNumber("291".to_string()), TransactionChange::NetworkFee(expected_total)]
        );
    }
}
