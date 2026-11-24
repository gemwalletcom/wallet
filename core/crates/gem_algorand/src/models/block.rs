use date_time::deserialize_datetime_from_seconds;
use primitives::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    pub block: Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    #[serde(rename = "ts", deserialize_with = "deserialize_datetime_from_seconds")]
    pub timestamp: DateTime,
    #[serde(rename = "txns", default)]
    pub transactions: Vec<SignedBlockTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBlockTransaction {
    #[serde(rename = "txn")]
    pub transaction: BlockTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTransaction {
    #[serde(rename = "type")]
    pub transaction_type: String,
    #[serde(rename = "snd")]
    pub sender: String,
    pub fee: Option<i64>,
    pub note: Option<String>,
    #[serde(rename = "amt")]
    pub amount: Option<i64>,
    #[serde(rename = "rcv")]
    pub receiver: Option<String>,
    #[serde(rename = "xaid")]
    pub asset_id: Option<u64>,
    #[serde(rename = "aamt")]
    pub asset_amount: Option<u64>,
    #[serde(rename = "arcv")]
    pub asset_receiver: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTransactionIds {
    #[serde(rename = "blockTxids")]
    pub transaction_ids: Vec<String>,
}
