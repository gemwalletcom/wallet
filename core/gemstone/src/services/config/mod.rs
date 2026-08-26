use std::sync::Arc;

use primitives::ConfigResponse;

use crate::api::{GemApiClient, GemApiError};

#[derive(Debug, uniffi::Object)]
pub struct GemConfigService {
    api: Arc<GemApiClient>,
}

#[uniffi::export]
impl GemConfigService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_config(&self) -> Result<ConfigResponse, GemApiError> {
        Ok(self.api.client.get_config().await?)
    }
}
