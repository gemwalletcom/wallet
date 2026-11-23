use date_time::deserialize_datetime_from_nanos;
use primitives::DateTime;
use serde::{Deserialize, Serialize};

use super::transaction::TransactionAction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub chunks: Vec<ChunkHeader>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub hash: String,
    pub height: u64,
    #[serde(deserialize_with = "deserialize_datetime_from_nanos")]
    pub timestamp: DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkHeader {
    pub chunk_hash: String,
    pub height_included: u64,
    pub tx_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub transactions: Vec<ChunkTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkTransaction {
    pub hash: String,
    pub signer_id: String,
    pub actions: Vec<TransactionAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub chain_id: String,
    pub sync_info: NodeSyncInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSyncInfo {
    pub latest_block_height: u64,
    pub syncing: bool,
}
