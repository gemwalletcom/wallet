use num_bigint::BigInt;

use crate::models::fee::EthereumFeeHistory;

impl EthereumFeeHistory {
    pub fn mock() -> Self {
        Self {
            reward: vec![vec!["0x5f5e100".to_string(), "0xbebc200".to_string()]],
            base_fee_per_gas: vec![BigInt::from(20_000_000_000u64)],
            gas_used_ratio: vec![0.5],
            oldest_block: 0x1234,
        }
    }
}
