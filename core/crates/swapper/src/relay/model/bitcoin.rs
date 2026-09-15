use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BitcoinStepData {
    pub psbt: String,
}
