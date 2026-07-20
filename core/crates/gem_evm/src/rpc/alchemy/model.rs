use num_bigint::BigUint;
use serde::Deserialize;
use serde_serializers::{deserialize_biguint_from_option_hex_str, deserialize_u64_from_str};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Transfer {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub(super) block_num: u64,
    pub(super) hash: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Transfers {
    pub(super) transfers: Vec<Transfer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenBalance {
    pub(super) contract_address: String,
    #[serde(deserialize_with = "deserialize_biguint_from_option_hex_str")]
    pub(super) token_balance: Option<BigUint>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenBalances {
    pub(super) token_balances: Vec<TokenBalance>,
}
