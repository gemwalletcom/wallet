use gem_evm::provider::preload_mapper::calculate_gas_limit_with_increase;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_option_bigint_from_str, serialize_option_bigint};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmStepData {
    pub to: String,
    pub data: Option<String>,
    pub value: String,
    #[serde(default, serialize_with = "serialize_option_bigint", deserialize_with = "deserialize_option_bigint_from_str")]
    pub gas: Option<BigInt>,
}

impl EvmStepData {
    pub fn gas_limit_with_buffer(&self) -> Option<String> {
        let gas = self.gas.clone()?;
        if gas <= BigInt::from(0) {
            return None;
        }
        Some(calculate_gas_limit_with_increase(gas).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::model::RelayQuoteResponse;

    #[test]
    fn test_evm_step() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("../testdata/quote_celo_native_to_bsc_usdt.json")).unwrap();
        let evm = response.get_evm_step().unwrap();

        assert_eq!(evm.gas, Some(BigInt::from(482935)));
        assert_eq!(evm.gas_limit_with_buffer().as_deref(), Some("724402"));
        assert!(response.get_tron_step().is_none());
    }
}
