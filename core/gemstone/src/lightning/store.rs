use std::sync::Arc;

use async_trait::async_trait;
use gem_spark::{LightningError, LightningLeaf, LightningPendingSend, LightningStore};
use primitives::WalletId;

use crate::services::error::GemServiceError;

#[uniffi::remote(Record)]
pub struct LightningLeaf {
    pub id: String,
    pub data: String,
}

#[uniffi::remote(Record)]
pub struct LightningPendingSend {
    pub transfer_id: String,
    pub invoice: String,
    pub amount_sats: Option<u64>,
    pub expires_at: u64,
}

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemLightningStore: Send + Sync {
    async fn get_leaves(&self, wallet_id: WalletId) -> Result<Vec<LightningLeaf>, GemServiceError>;
    async fn update_leaves(&self, wallet_id: WalletId, leaves: Vec<LightningLeaf>, delete_ids: Vec<String>) -> Result<(), GemServiceError>;
    async fn get_pending_sends(&self, wallet_id: WalletId) -> Result<Vec<LightningPendingSend>, GemServiceError>;
    async fn add_pending_send(&self, wallet_id: WalletId, send: LightningPendingSend) -> Result<(), GemServiceError>;
    async fn delete_pending_send(&self, wallet_id: WalletId, transfer_id: String) -> Result<(), GemServiceError>;
}

pub(crate) struct WalletLightningStore {
    store: Arc<dyn GemLightningStore>,
    wallet_id: WalletId,
}

impl WalletLightningStore {
    pub(crate) fn new(store: Arc<dyn GemLightningStore>, wallet_id: WalletId) -> Self {
        Self { store, wallet_id }
    }
}

fn storage_error(error: GemServiceError) -> LightningError {
    LightningError::Storage(error.to_string())
}

#[async_trait]
impl LightningStore for WalletLightningStore {
    async fn get_leaves(&self) -> Result<Vec<LightningLeaf>, LightningError> {
        self.store.get_leaves(self.wallet_id.clone()).await.map_err(storage_error)
    }

    async fn update_leaves(&self, leaves: Vec<LightningLeaf>, delete_ids: Vec<String>) -> Result<(), LightningError> {
        self.store.update_leaves(self.wallet_id.clone(), leaves, delete_ids).await.map_err(storage_error)
    }

    async fn get_pending_sends(&self) -> Result<Vec<LightningPendingSend>, LightningError> {
        self.store.get_pending_sends(self.wallet_id.clone()).await.map_err(storage_error)
    }

    async fn add_pending_send(&self, send: LightningPendingSend) -> Result<(), LightningError> {
        self.store.add_pending_send(self.wallet_id.clone(), send).await.map_err(storage_error)
    }

    async fn delete_pending_send(&self, transfer_id: String) -> Result<(), LightningError> {
        self.store.delete_pending_send(self.wallet_id.clone(), transfer_id).await.map_err(storage_error)
    }
}
