use num_bigint::BigUint;

use crate::UTXO;

impl UTXO {
    pub fn mock_with_value(value: u64) -> Self {
        UTXO {
            transaction_id: "transaction".to_string(),
            vout: 0,
            value: BigUint::from(value),
            address: "sender".to_string(),
        }
    }
}
