use primitives::{TransactionState, TransactionUpdate};

use crate::models::{Digest, STATUS_FAILURE, STATUS_SUCCESS};

pub fn map_transaction_status(transaction: Digest) -> TransactionUpdate {
    let state = match transaction.effects.status.status.as_str() {
        STATUS_SUCCESS => TransactionState::Confirmed,
        STATUS_FAILURE => TransactionState::Reverted,
        _ => TransactionState::Pending,
    };
    TransactionUpdate::new_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Effect, GasObject, GasUsed, Owner, Status};
    use num_bigint::BigUint;

    #[test]
    fn test_map_transaction_status() {
        let digest = Digest {
            digest: "test".to_string(),
            effects: Effect {
                gas_used: GasUsed {
                    computation_cost: BigUint::from(1000u32),
                    storage_cost: BigUint::from(500u32),
                    storage_rebate: BigUint::from(100u32),
                    non_refundable_storage_fee: BigUint::from(0u32),
                },
                status: Status {
                    status: STATUS_SUCCESS.to_string(),
                },
                gas_object: GasObject {
                    owner: Owner::String("0x123".to_string()),
                },
            },
            move_call_packages: Vec::new(),
            balance_changes: None,
            events: vec![],
            timestamp_ms: 1234567890,
        };

        let update = map_transaction_status(digest);
        assert_eq!(update.state, TransactionState::Confirmed);
    }
}
