use std::str::FromStr;

use alloy_primitives::Address;
use num_bigint::BigUint;

use crate::constants::TOKEN_TRANSFER_GAS_LIMIT;
use crate::rpc::{
    mapper::TRANSFER_TOPIC,
    model::{Log, Transaction, TransactionReceipt},
};

use super::TEST_TRANSACTION_ID;

impl Log {
    pub fn mock_erc20_transfer(contract: &str, from: &str, to: &str, value: u64) -> Self {
        Self {
            address: contract.to_string(),
            topics: vec![TRANSFER_TOPIC.to_string(), address_topic(from), address_topic(to)],
            data: format!("0x{value:064x}"),
            transaction_hash: None,
        }
    }
}

fn address_topic(address: &str) -> String {
    Address::from_str(address).unwrap().into_word().to_string()
}

impl Transaction {
    pub fn mock_erc20_transfer(contract: &str) -> Self {
        Self {
            hash: TEST_TRANSACTION_ID.to_string(),
            from: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            gas: TOKEN_TRANSFER_GAS_LIMIT,
            input: "0xa9059cbb".to_string(),
            to: Some(contract.to_string()),
            value: BigUint::from(0u8),
            calls: None,
        }
    }
}

impl TransactionReceipt {
    pub fn mock_with_log(log: Log) -> Self {
        Self {
            gas_used: BigUint::from(50_000u32),
            effective_gas_price: BigUint::from(20_000_000_000u64),
            l1_fee: None,
            logs: vec![log],
            status: "0x1".to_string(),
            block_hash: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            block_number: 1000,
            fee_token: None,
        }
    }
}
