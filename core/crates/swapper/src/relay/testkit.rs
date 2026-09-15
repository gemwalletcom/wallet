use std::sync::Arc;

use gem_client::testkit::MockClient;
use num_bigint::BigInt;
use primitives::Chain;

use super::{
    Relay,
    model::{CurrencyAmount, EvmStepData, QuoteDetails, RelayChainInfo, RelayProtocol, RelayProtocolV2, RelayQuoteResponse, RelayRequest, RelayStatus, Step, StepData, StepItem},
};
use crate::{Quote, alien::mock::ProviderMock};

pub const TEST_ROUTER_ADDRESS: &str = "0xCcC88a9d1B4ED6b0EABA998850414b24f1c315bE";
pub const TEST_QUOTE_VALUE: &str = "40000000000000000000";
pub const TEST_ZERO_ALLOWANCE: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
pub const TEST_SUFFICIENT_ALLOWANCE: &str = "0x0000000000000000000000000000000000000000000000022b1c8c1227a00000";

impl Relay<MockClient> {
    pub fn mock_with_allowance(allowance_result: &str) -> Self {
        Self::new_with_client(
            MockClient::new(),
            Arc::new(ProviderMock::new(format!(r#"{{"id":1,"jsonrpc":"2.0","result":"{allowance_result}"}}"#))),
        )
    }

    pub fn mock_with_chains(chains: &'static str) -> Self {
        Self::new_with_client(MockClient::new().with_get(|_| Ok(chains.as_bytes().to_vec())), Arc::new(ProviderMock::new(String::new())))
    }

    pub fn mock_with_tron_allowance(allowance: &str) -> Self {
        Self::new_with_client(MockClient::new(), Arc::new(ProviderMock::new(format!(r#"{{"constant_result":["{allowance}"]}}"#))))
    }
}

pub fn mock_quote(chain: Chain) -> Quote {
    let mut quote = Quote::mock(chain, None);
    quote.from_value = TEST_QUOTE_VALUE.parse().unwrap();
    quote.request.wallet_address = "0x1085c5f70F7F7591D97da281A64688385455c2bD".to_string();
    quote
}

pub fn mock_quote_response() -> RelayQuoteResponse {
    RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction("deposit", TEST_ROUTER_ADDRESS, "0", "0xf9e4bab4")])
}

impl RelayQuoteResponse {
    pub fn mock_with_steps(steps: Vec<Step>) -> Self {
        Self {
            steps,
            details: QuoteDetails::mock(),
        }
    }
}

impl QuoteDetails {
    pub fn mock() -> Self {
        Self {
            currency_out: CurrencyAmount { amount: "0".to_string() },
            time_estimate: None,
            slippage_tolerance: None,
        }
    }
}

impl EvmStepData {
    pub fn mock_with_gas(gas: Option<u64>) -> Self {
        Self {
            to: "0xrouter".to_string(),
            data: None,
            value: "0".to_string(),
            gas: gas.map(BigInt::from),
        }
    }
}

impl RelayRequest {
    pub fn mock_with_status(status: RelayStatus) -> Self {
        Self { status, data: None }
    }
}

impl RelayChainInfo {
    pub fn mock(id: u64, depository: Option<&str>, solver_addresses: &[&str]) -> Self {
        Self {
            id,
            solver_addresses: solver_addresses.iter().map(|address| address.to_string()).collect(),
            protocol: Some(RelayProtocol {
                v2: Some(RelayProtocolV2 {
                    depository: depository.map(str::to_string),
                }),
            }),
        }
    }
}

impl Step {
    pub fn mock_transaction(id: &str, to: &str, value: &str, data: &str) -> Self {
        Self::mock_transaction_with_gas(id, to, value, data, None)
    }

    pub fn mock_transaction_with_gas(id: &str, to: &str, value: &str, data: &str, gas: Option<u64>) -> Self {
        Self {
            id: id.to_string(),
            kind: "transaction".to_string(),
            items: Some(vec![StepItem {
                data: Some(StepData::Evm(EvmStepData {
                    to: to.to_string(),
                    data: Some(data.to_string()),
                    value: value.to_string(),
                    gas: gas.map(BigInt::from),
                })),
            }]),
        }
    }

    pub fn mock_empty(id: &str, kind: &str) -> Self {
        Self {
            id: id.to_string(),
            kind: kind.to_string(),
            items: None,
        }
    }
}
