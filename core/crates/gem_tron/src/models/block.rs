use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronBlock {
    #[serde(rename = "block_header")]
    pub block_header: TronHeaderRawData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronHeaderRawData {
    pub raw_data: TronHeader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronHeader {
    pub number: u64,
    pub version: u64,
    #[serde(rename = "txTrieRoot")]
    pub tx_trie_root: String,
    pub witness_address: String,
    #[serde(rename = "parentHash")]
    pub parent_hash: String,
    pub timestamp: u64,
}
