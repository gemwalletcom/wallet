use crate::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionStateRequest {
    pub id: String,
    pub sender_address: String,
    pub created_at: DateTime,
    pub block_number: u64,
}
