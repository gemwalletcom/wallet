use num_bigint::BigInt;

use super::model::{
    CurrencyAmount, EvmStepData, QuoteDetails, RelayChainInfo, RelayProtocol, RelayProtocolV2, RelayQuoteResponse, RelayRequest, RelayStatus, Step, StepData, StepItem,
};

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
