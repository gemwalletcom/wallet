use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_option_biguint_from_str, serialize_option_biguint};

use crate::AssetId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AssetAddress {
    pub asset_id: AssetId,
    pub address: String,
    #[serde(default, serialize_with = "serialize_option_biguint", deserialize_with = "deserialize_option_biguint_from_str")]
    pub value: Option<BigUint>,
}

impl AssetAddress {
    pub fn new(asset_id: AssetId, address: String, value: Option<BigUint>) -> Self {
        Self { asset_id, address, value }
    }
}
