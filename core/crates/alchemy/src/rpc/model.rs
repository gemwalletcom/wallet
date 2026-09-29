use num_bigint::BigUint;
use primitives::Chain;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_option_hex_str, deserialize_u64_from_str};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransferCategory {
    External,
    Internal,
    Erc20,
    Erc721,
    Erc1155,
    Specialnft,
}

pub const INTERNAL_TRANSFER_CHAINS: [Chain; 3] = [Chain::Ethereum, Chain::Polygon, Chain::Base];

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub block_num: u64,
    pub hash: String,
    pub from: String,
    pub category: TransferCategory,
    pub raw_contract: RawContract,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct RawContract {
    #[serde(deserialize_with = "deserialize_biguint_from_option_hex_str")]
    pub value: Option<BigUint>,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfers {
    pub transfers: Vec<Transfer>,
    pub page_key: Option<String>,
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
