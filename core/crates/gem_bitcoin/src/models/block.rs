use serde::{Deserialize, Serialize};

use crate::models::Transaction;

type Int = u64;

// Domain models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinBlock {
    pub previous_block_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinNodeInfo {
    pub blockbook: BitcoinBlockbook,
    pub backend: BitcoinBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinBlockbook {
    pub in_sync: bool,
    pub last_block_time: String,
    pub best_height: Int,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinBackend {
    pub blocks: Option<Int>,
    pub chain: Option<String>,
    pub consensus: Option<Consensus>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Consensus {
    pub chaintip: String,
}

// RPC models
#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
    pub blockbook: Blockbook,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Blockbook {
    #[serde(rename = "bestHeight")]
    pub best_height: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub page: u64,
    pub total_pages: u64,
    pub txs: Vec<Transaction>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_info_deserializes_without_blocks() {
        let info: BitcoinNodeInfo = serde_json::from_str(include_str!("../../testdata/node_info_no_blocks.json")).unwrap();
        assert_eq!(info.backend.blocks, None);
        assert_eq!(info.backend.consensus.unwrap().chaintip, "c2d6d0b4");
    }
}
