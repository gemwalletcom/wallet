use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub sequence: u64,
    pub balances: Vec<Balance>,
    pub thresholds: Thresholds,
    pub signers: Vec<Signer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub med_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signer {
    pub key: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountEmpty {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub balance: String,
    pub asset_type: String,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,
}
