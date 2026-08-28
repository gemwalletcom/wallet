use crate::UInt64;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionStateRequest {
    pub id: String,
    pub sender_address: String,
    pub created_at: DateTime<Utc>,
    pub block_number: UInt64,
}
