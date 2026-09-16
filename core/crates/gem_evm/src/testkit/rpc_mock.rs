use std::str::FromStr;

use alloy_primitives::Address;
use num_bigint::BigUint;

use gem_client::testkit::MockClient;
use gem_jsonrpc::{JsonRpcClient, testkit::mock_jsonrpc_client};
use primitives::EVMChain;
use serde_json::Value;

use crate::constants::TOKEN_TRANSFER_GAS_LIMIT;
use crate::method;
use crate::rpc::{
    EthereumClient,
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
    pub fn mock() -> Self {
        Self {
            hash: TEST_TRANSACTION_ID.to_string(),
            from: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            gas: 21_000,
            input: "0x".to_string(),
            to: None,
            value: BigUint::from(0u8),
            calls: None,
        }
    }

    pub fn mock_erc20_transfer(contract: &str) -> Self {
        Self {
            gas: TOKEN_TRANSFER_GAS_LIMIT,
            input: "0xa9059cbb".to_string(),
            to: Some(contract.to_string()),
            ..Self::mock()
        }
    }
}

impl TransactionReceipt {
    pub fn mock() -> Self {
        Self {
            gas_used: BigUint::from(21_000u32),
            effective_gas_price: BigUint::from(20_000_000_000u64),
            l1_fee: None,
            logs: vec![],
            status: "0x1".to_string(),
            block_hash: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            block_number: 1000,
            fee_token: None,
        }
    }

    pub fn mock_with_log(log: Log) -> Self {
        Self {
            gas_used: BigUint::from(50_000u32),
            logs: vec![log],
            ..Self::mock()
        }
    }
}

impl EthereumClient<MockClient> {
    pub fn mock(chain: EVMChain) -> Self {
        EthereumClient::new(JsonRpcClient::new(MockClient::new()), chain)
    }

    pub fn mock_with_code(code: &str) -> Self {
        let code = code.to_string();
        let client = mock_jsonrpc_client(move |request_method, _| match request_method {
            method::ETH_GET_CODE => Ok(Value::from(code.clone())),
            _ => Ok(Value::Null),
        });
        EthereumClient::new(client, EVMChain::Ethereum)
    }
}
