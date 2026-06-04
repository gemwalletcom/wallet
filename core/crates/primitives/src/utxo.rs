use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct UTXO {
    pub transaction_id: String,
    pub vout: i32,
    pub value: String,
    pub address: String,
}

impl UTXO {
    pub fn value_u64(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        self.value.parse().map_err(|_| format!("invalid UTXO amount: {}", self.value).into())
    }
}
