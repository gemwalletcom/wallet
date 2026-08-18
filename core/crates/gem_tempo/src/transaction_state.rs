use gem_evm::provider::transaction_state_mapper::map_transaction_status_with_fee;
use gem_evm::rpc::model::TransactionReceipt;
use primitives::TransactionUpdate;

use crate::fee::scale_fee_to_token_units;

pub fn map_transaction_status(receipt: &TransactionReceipt) -> TransactionUpdate {
    map_transaction_status_with_fee(receipt, scale_fee_to_token_units(receipt.get_fee().into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::{BigInt, BigUint};
    use primitives::{TransactionChange, TransactionState};

    #[test]
    fn test_map_transaction_status_scales_fee_to_token_units() {
        let receipt = TransactionReceipt {
            gas_used: BigUint::from(471_789u64),
            effective_gas_price: BigUint::from(1_260_212_000u64),
            l1_fee: None,
            logs: vec![],
            status: "0x1".to_string(),
            block_hash: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            block_number: 291,
            fee_token: None,
        };

        let result = map_transaction_status(&receipt);

        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(
            result.changes,
            vec![TransactionChange::BlockNumber("291".to_string()), TransactionChange::NetworkFee(BigInt::from(595u64))]
        );
    }
}
