use async_trait::async_trait;

use crate::error::LightningError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningLeaf {
    pub id: String,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningPendingSend {
    pub transfer_id: String,
    pub invoice: String,
    pub amount_sats: Option<u64>,
    pub expires_at: u64,
}

#[async_trait]
pub trait LightningStore: Send + Sync {
    async fn get_leaves(&self) -> Result<Vec<LightningLeaf>, LightningError>;
    async fn update_leaves(&self, leaves: Vec<LightningLeaf>, delete_ids: Vec<String>) -> Result<(), LightningError>;
    async fn get_pending_sends(&self) -> Result<Vec<LightningPendingSend>, LightningError>;
    async fn add_pending_send(&self, send: LightningPendingSend) -> Result<(), LightningError>;
    async fn delete_pending_send(&self, transfer_id: String) -> Result<(), LightningError>;
}
