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

    #[test]
    fn test_gas_limit_with_buffer() {
        assert_eq!(EvmStepData::mock_with_gas(Some(100_000)).gas_limit_with_buffer().as_deref(), Some("150000"));
        assert_eq!(EvmStepData::mock_with_gas(Some(0)).gas_limit_with_buffer(), None);
        assert_eq!(EvmStepData::mock_with_gas(None).gas_limit_with_buffer(), None);
    }
}
