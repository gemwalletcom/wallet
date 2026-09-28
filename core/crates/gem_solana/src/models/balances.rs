use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SolanaBalance {
    pub value: u64,
}
