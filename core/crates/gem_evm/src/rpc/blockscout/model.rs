use num_bigint::BigUint;
use serde::Deserialize;
use serde_serializers::deserialize_biguint_from_str;

#[derive(Debug, Deserialize)]
pub(super) struct Items<T> {
    pub(super) items: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Transaction {
    pub(super) hash: String,
    pub(super) block_number: u64,
}

#[derive(Debug, Deserialize)]
pub(super) struct TokenTransfer {
    pub(super) transaction_hash: String,
    pub(super) block_number: u64,
}

#[derive(Debug, Deserialize)]
pub(super) struct TokenBalance {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub(super) value: BigUint,
    pub(super) token: Token,
}

#[derive(Debug, Deserialize)]
pub(super) struct Token {
    pub(super) address_hash: String,
    pub(super) reputation: Option<String>,
    #[serde(rename = "type")]
    pub(super) token_type: String,
}
