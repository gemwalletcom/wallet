use std::sync::Arc;

use primitives::AuthNonce;

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemAuthService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemAuthService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_nonce(&self) -> Result<AuthNonce, GemApiError> {
        Ok(self.api.client.get_auth_nonce().await?)
    }
}
