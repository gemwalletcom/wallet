use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
    pub storage_usage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAccessKey {
    pub nonce: i64,
    pub permission: AccountAccessKeyPermission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountAccessKeyPermission {
    FullAccess,
    FunctionCall(FunctionCallPermission),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallPermission {
    pub allowance: Option<String>,
    pub receiver_id: String,
    pub method_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAccessKeyEntry {
    pub public_key: String,
    pub access_key: AccountAccessKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAccessKeyList {
    pub keys: Vec<AccountAccessKeyEntry>,
}
