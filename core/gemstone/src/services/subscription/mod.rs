pub mod rules;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use crate::services::wallet::GemWalletStore;

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(uniffi::Object)]
pub struct GemSubscriptionService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemWalletStore>,
}

#[uniffi::export]
impl GemSubscriptionService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, store: Arc<dyn GemWalletStore>) -> Self {
        Self { api, store }
    }

    pub async fn sync(&self) -> Result<bool, GemServiceError> {
        let local = rules::wallet_subscriptions(&self.store.get_wallets().await?);
        let remote = self.api.client.get_subscriptions().await.map_err(GemApiError::from)?;
        let changes = rules::subscription_changes(local, remote);
        if changes.is_empty() {
            return Ok(false);
        }
        if !changes.to_add.is_empty() {
            self.api.client.add_subscriptions(changes.to_add).await.map_err(GemApiError::from)?;
        }
        if !changes.to_delete.is_empty() {
            self.api.client.delete_subscriptions(changes.to_delete).await.map_err(GemApiError::from)?;
        }
        Ok(true)
    }
}
