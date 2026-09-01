use num_bigint::BigUint;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, serialize_biguint};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct UTXO {
    pub transaction_id: String,
    pub vout: i32,
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
    pub address: String,
}

impl UTXO {
    pub fn value_u64(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        self.value.to_u64().ok_or_else(|| format!("UTXO amount is too large: {}", self.value).into())
    }
}
