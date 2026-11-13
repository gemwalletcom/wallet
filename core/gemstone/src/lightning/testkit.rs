use async_trait::async_trait;
use gem_spark::LightningPendingSend;
use primitives::WalletId;

use super::store::GemLightningStore;
use crate::services::error::GemServiceError;

pub struct NoopLightningStore;

#[async_trait]
impl GemLightningStore for NoopLightningStore {
    async fn get_pending_sends(&self, _wallet_id: WalletId) -> Result<Vec<LightningPendingSend>, GemServiceError> {
        Ok(vec![])
    }

    async fn add_pending_send(&self, _wallet_id: WalletId, _send: LightningPendingSend) -> Result<(), GemServiceError> {
        Ok(())
    }

    async fn delete_pending_send(&self, _wallet_id: WalletId, _transfer_id: String) -> Result<(), GemServiceError> {
        Ok(())
    }
}
