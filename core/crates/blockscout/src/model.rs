use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_serializers::deserialize_biguint_from_str;

#[derive(Debug, Deserialize)]
pub(super) struct Items<T> {
    pub(super) items: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub(super) struct NftPage {
    pub(super) items: Vec<NftItem>,
    pub(super) next_page_params: Option<NftPageParams>,
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
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub icon_url: Option<String>,
    pub reputation: Option<String>,
    #[serde(rename = "type")]
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct NftItem {
    pub id: String,
    pub token: Token,
}

#[derive(Debug, Deserialize)]
pub struct NftInstance {
    pub id: String,
    pub image_url: Option<String>,
    pub metadata: Option<NftMetadata>,
    pub token: Token,
}

#[derive(Debug, Deserialize)]
pub struct NftMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub attributes: Option<Vec<NftAttribute>>,
}

#[derive(Debug, Deserialize)]
pub struct NftAttribute {
    pub trait_type: Option<String>,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftPageParams {
    pub token_type: String,
    pub token_contract_address_hash: String,
    pub token_id: String,
    pub items_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NftQuery {
    #[serde(rename = "type")]
    pub token_types: &'static str,
    #[serde(flatten)]
    pub page: Option<NftPageParams>,
}

impl NftQuery {
    pub fn new(page: Option<NftPageParams>) -> Self {
        Self { token_types: "ERC-721,ERC-1155", page }
    }
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
