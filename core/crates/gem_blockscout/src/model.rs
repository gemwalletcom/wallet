use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

#[derive(Debug, Deserialize)]
pub(super) struct Items<T> {
    pub(super) items: Vec<T>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub hash: String,
    pub block_number: u64,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct TokenTransfer {
    pub transaction_hash: String,
    pub block_number: u64,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct TokenBalance {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
    pub token: Token,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Token {
    pub address_hash: String,
    pub reputation: Option<String>,
    #[serde(rename = "type")]
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PageQuery {
    pub sort: &'static str,
    pub order: &'static str,
    pub items_count: usize,
}

impl PageQuery {
    pub fn newest(items_count: usize) -> Self {
        Self {
            sort: "block_number",
            order: "desc",
            items_count,
        }
    }
}
