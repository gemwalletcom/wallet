use std::sync::Arc;

use primitives::{Chain, StakeValidator};

use crate::api::{GemApiError, GemStaticApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemStaticAssetsService {
    api: Arc<GemStaticApiClient>,
}

#[uniffi::export]
impl GemStaticAssetsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemStaticApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_validators(&self, chain: Chain) -> Result<Vec<StakeValidator>, GemApiError> {
        Ok(self.api.client.get_validators(chain).await?)
    }
}
