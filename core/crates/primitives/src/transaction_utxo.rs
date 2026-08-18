use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, serialize_biguint};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Sendable, Equatable, Hashable")]
pub struct TransactionUtxoInput {
    pub address: String, // Coinbase / OP_Return will be filtered
    #[serde(deserialize_with = "deserialize_biguint_from_str", serialize_with = "serialize_biguint")]
    pub value: BigUint,
}

impl TransactionUtxoInput {
    pub fn new(address: String, value: BigUint) -> Self {
        Self { address, value }
    }
}
