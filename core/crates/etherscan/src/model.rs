use std::error::Error;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_serializers::{deserialize_f64_from_str, deserialize_u64_from_str};

const STATUS_OK: &str = "1";

#[derive(Debug, Deserialize)]
pub struct EtherscanResponse {
    status: String,
    message: String,
    result: Value,
}

impl EtherscanResponse {
    pub fn into_result<T: DeserializeOwned>(self) -> Result<T, Box<dyn Error + Send + Sync>> {
        if self.status != STATUS_OK {
            return Err(format!("Etherscan {}: {}", self.message, self.result).into());
        }
        Ok(serde_json::from_value(self.result)?)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GasOracle {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub last_block: u64,
    #[serde(rename = "suggestBaseFee", deserialize_with = "deserialize_f64_from_str")]
    pub suggest_base_fee: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub propose_gas_price: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fast_gas_price: f64,
    #[serde(rename = "gasUsedRatio")]
    pub gas_used_ratio: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_result() {
        let response: EtherscanResponse = serde_json::from_str(include_str!("../testdata/gas_oracle.json")).unwrap();
        assert_eq!(
            response.into_result::<GasOracle>().unwrap(),
            GasOracle {
                last_block: 26_137_033,
                suggest_base_fee: 0.122955261,
                propose_gas_price: 0.123955261,
                fast_gas_price: 0.165642084,
                gas_used_ratio: "0.736964583333333,0.520348904717086,0.303028887877443,0.657702966666667,0.571717066666667".to_string(),
            }
        );

        let response: EtherscanResponse = serde_json::from_str(include_str!("../testdata/error.json")).unwrap();
        assert_eq!(response.into_result::<GasOracle>().unwrap_err().to_string(), "Etherscan NOTOK: \"Invalid API Key (#err2)|1\"");
    }
}
