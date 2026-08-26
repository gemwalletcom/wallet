use std::sync::Arc;

use primitives::WalletConfigurationResult;

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemWalletConfigurationService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemWalletConfigurationService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_configuration(&self, wallet_id: String) -> Result<WalletConfigurationResult, GemApiError> {
        Ok(self.api.client.get_wallet_configuration(wallet_id).await?)
    }
}
