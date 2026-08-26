use std::sync::Arc;

use primitives::TransactionsResponse;

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemTransactionsService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemTransactionsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_transactions(&self, wallet_id: String, asset_id: Option<String>, from_timestamp: u64) -> Result<TransactionsResponse, GemApiError> {
        Ok(self.api.client.get_transactions(wallet_id, asset_id, from_timestamp).await?)
    }

    pub async fn get_assets_list(&self, wallet_id: String, from_timestamp: u64) -> Result<Vec<String>, GemApiError> {
        Ok(self.api.client.get_assets_list(wallet_id, from_timestamp).await?)
    }
}
