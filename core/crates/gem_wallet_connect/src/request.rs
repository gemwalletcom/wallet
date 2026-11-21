use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_option_hex_str, deserialize_option_u64_from_str_or_int};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WCEthereumTransaction {
    #[serde(default, deserialize_with = "deserialize_option_u64_from_str_or_int")]
    pub chain_id: Option<u64>,
    pub from: String,
    pub to: String,
    #[serde(default, deserialize_with = "deserialize_biguint_from_option_hex_str")]
    pub value: Option<BigUint>,
    pub gas: Option<String>,
    pub gas_limit: Option<String>,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub nonce: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletConnectRequest {
    pub topic: String,
    pub method: String,
    pub params: String,
    pub chain_id: Option<String>,
    pub domain: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_wc_ethereum_transaction() {
        let value = |field: &str| {
            serde_json::from_str::<WCEthereumTransaction>(&format!(r#"{{"from":"0xsender","to":"0xrouter"{field}}}"#))
                .map(|transaction| transaction.value)
                .map_err(|error| error.to_string())
        };

        assert_eq!(value(""), Ok(None));
        assert_eq!(value(r#","value":null"#), Ok(None));
        assert_eq!(value(r#","value":"0x""#), Ok(Some(BigUint::ZERO)));
        assert_eq!(value(r#","value":"0x2386f26fc10000""#), Ok(Some(BigUint::from(10_000_000_000_000_000u64))));
        assert_eq!(value(r#","value":"0xzz""#), Err("Invalid hex string: 0xzz at line 1 column 50".to_string()));
    }
}
