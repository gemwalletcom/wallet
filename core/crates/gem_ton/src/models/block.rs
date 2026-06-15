use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chainhead {
    pub last: BlockInfo,
    #[serde(rename = "first")]
    pub first: BlockInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub seqno: u64,
    pub root_hash: String,
}
