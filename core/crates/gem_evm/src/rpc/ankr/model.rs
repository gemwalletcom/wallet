use num_bigint::BigUint;
use serde::Deserialize;
use serde_serializers::{deserialize_biguint_from_str, deserialize_u64_from_str_or_int};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Transaction {
    pub(super) hash: String,
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub(super) timestamp: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Transactions {
    pub(super) transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenTransfer {
    pub(super) transaction_hash: String,
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub(super) timestamp: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenTransfers {
    pub(super) transfers: Vec<TokenTransfer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenBalances {
    pub(super) assets: Vec<TokenBalance>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenBalance {
    pub(super) contract_address: Option<String>,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub(super) balance_raw_integer: BigUint,
}
