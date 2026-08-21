use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    #[serde(default)]
    pub chunks: Vec<ChunkHeader>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub hash: String,
    pub height: u64,
    #[serde(default)]
    pub timestamp: u64,
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
