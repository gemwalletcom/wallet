use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_bigint_from_str, deserialize_option_bigint_or_none, deserialize_u64_from_str, serialize_bigint, serialize_option_bigint};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotAccountBalance {
    #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint_from_str")]
    pub free: BigInt,
    #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint_from_str")]
    pub reserved: BigInt,
    #[serde(default, serialize_with = "serialize_option_bigint", deserialize_with = "deserialize_option_bigint_or_none")]
    pub frozen: Option<BigInt>,
    #[serde(default, serialize_with = "serialize_option_bigint", deserialize_with = "deserialize_option_bigint_or_none")]
    pub transferable: Option<BigInt>,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub nonce: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_balance_info() {
        let modern: PolkadotAccountBalance = serde_json::from_str(r#"{"nonce":"48","free":"31415926535","reserved":"0","frozen":"31415926535","transferable":"0"}"#).unwrap();

        assert_eq!(modern.free, BigInt::from(31415926535_u64));
        assert_eq!(modern.reserved, BigInt::from(0));
        assert_eq!(modern.frozen, Some(BigInt::from(31415926535_u64)));
        assert_eq!(modern.transferable, Some(BigInt::from(0)));
        assert_eq!(modern.nonce, 48);

        let old_runtime: PolkadotAccountBalance =
            serde_json::from_str(r#"{"nonce":"1","free":"1000","reserved":"100","frozen":"frozen does not exist for this runtime"}"#).unwrap();

        assert_eq!(old_runtime.free, BigInt::from(1000));
        assert_eq!(old_runtime.reserved, BigInt::from(100));
        assert_eq!(old_runtime.frozen, None);
        assert_eq!(old_runtime.transferable, None);
    }
}
