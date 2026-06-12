use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotNodeVersion {
    pub chain: String,
}
