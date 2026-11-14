use serde::{Deserialize, Serialize};

use crate::error::LightningError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightningSend {
    pub transfer_id: String,
    pub invoice: String,
    pub amount_sats: Option<u64>,
    pub expires_at: u64,
}

pub trait LightningStore: Send + Sync {
    fn get_sends(&self) -> Result<Vec<LightningSend>, LightningError>;
    fn add_send(&self, send: LightningSend) -> Result<(), LightningError>;
    fn delete_send(&self, transfer_id: &str) -> Result<(), LightningError>;
}
