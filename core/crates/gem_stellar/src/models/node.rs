#[cfg(feature = "rpc")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub ingest_latest_ledger: i32,
    pub network_passphrase: String,
}
