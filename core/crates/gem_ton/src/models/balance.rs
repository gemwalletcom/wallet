use std::collections::HashMap;

use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, deserialize_option_u64_from_str_or_int};

#[derive(Debug, Clone, Deserialize)]
pub struct JettonMastersResponse {
    pub jetton_masters: Vec<JettonMaster>,
    pub metadata: HashMap<String, JettonMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JettonMaster {
    pub address: String,
    pub jetton_content: JettonContent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JettonContent {
    pub name: Option<String>,
    pub symbol: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_u64_from_str_or_int")]
    pub decimals: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JettonMetadata {
    pub token_info: Vec<JettonTokenInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JettonTokenInfo {
    #[serde(default)]
    pub valid: bool,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub extra: Option<JettonTokenInfoExtra>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JettonTokenInfoExtra {
    #[serde(default, deserialize_with = "deserialize_option_u64_from_str_or_int")]
    pub decimals: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jetton {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonWalletsResponse {
    pub jetton_wallets: Vec<JettonWallet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JettonWallet {
    pub address: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub balance: BigUint,
    pub jetton: String,
}
