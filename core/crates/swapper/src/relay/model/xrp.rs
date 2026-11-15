use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, serialize_biguint};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct XrpStepData {
    pub to: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str", serialize_with = "serialize_biguint")]
    pub amount: BigUint,
    pub memo: String,
}
