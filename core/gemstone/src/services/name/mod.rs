use std::sync::Arc;

use primitives::name::NameRecord;
use primitives::{AddressName, ChainAddress};

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemNameService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemNameService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn resolve(&self, name: String, chain: String) -> Result<Option<NameRecord>, GemApiError> {
        Ok(self.api.client.get_name_record(name, chain).await?)
    }

    pub async fn get_address_names(&self, requests: Vec<ChainAddress>) -> Result<Vec<AddressName>, GemApiError> {
        Ok(self.api.client.get_address_names(requests).await?)
    }
}
