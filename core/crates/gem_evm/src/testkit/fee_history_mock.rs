use num_bigint::BigInt;

use crate::models::fee::EthereumFeeHistory;

impl EthereumFeeHistory {
    pub fn mock() -> Self {
        Self {
            reward: vec![vec!["0xbebc200".to_string(), "0x11e1a300".to_string()]],
            base_fee_per_gas: vec![BigInt::from(20_000_000_000u64)],
            gas_used_ratio: vec![0.5],
            oldest_block: 0x1234,
        }
    }

    pub fn mock_ethereum() -> Self {
        Self {
            reward: vec![
                vec!["0x31e7fe5d".to_string(), "0x3b9aca04".to_string()],
                vec!["0x18bf8474".to_string(), "0x3b9aca00".to_string()],
                vec!["0x32324960".to_string(), "0x3b9aca00".to_string()],
                vec!["0x7bf60c0".to_string(), "0x31e7fe5d".to_string()],
                vec!["0x29b92700".to_string(), "0x39fbe24e".to_string()],
            ],
            base_fee_per_gas: vec![
                BigInt::from(2618877110u64),
                BigInt::from(2600645117u64),
                BigInt::from(2474034920u64),
                BigInt::from(2495024366u64),
                BigInt::from(2624620404u64),
                BigInt::from(2541053471u64),
            ],
            gas_used_ratio: vec![0.4787648265769147, 0.30434244444444447, 0.5349411706429458, 0.707018, 0.37411107986145914],
            oldest_block: 22832041,
        }
    }
}
