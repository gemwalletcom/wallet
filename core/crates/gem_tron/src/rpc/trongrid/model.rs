use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data<T> {
    pub data: T,
    #[serde(default)]
    pub meta: Option<Meta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronGridAccount {
    #[serde(default)]
    pub trc20: Vec<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TronGridTransaction {
    #[serde(alias = "txID")]
    pub transaction_id: String,
    pub block_timestamp: u64,
}
