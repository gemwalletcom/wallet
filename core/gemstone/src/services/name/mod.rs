pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::name::NameRecord;
use primitives::{AddressName, Chain, ChainAddress};

use crate::api::{GemApiError, GemDeviceApiClient};

pub use store::GemAddressStore;

#[derive(uniffi::Object)]
pub struct GemNameService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemAddressStore>,
}

#[uniffi::export]
impl GemNameService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, store: Arc<dyn GemAddressStore>) -> Self {
        Self { api, store }
    }

    pub fn is_name_supported(&self, name: String) -> bool {
        rules::is_name_supported(&name)
    }

    pub async fn get_name_record(&self, name: String, chain: Chain) -> Result<Option<NameRecord>, GemServiceError> {
        Ok(self.api.client.get_name_record(name, chain.to_string()).await.map_err(GemApiError::from)?)
    }

    pub async fn get_address_names(&self, requests: Vec<ChainAddress>) -> Result<Vec<AddressName>, GemServiceError> {
        let requests = rules::unique_requests(requests);
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let mut cached = Vec::new();
        let mut missing = Vec::new();
        for request in requests {
            match self.store.get_address_name(request.chain, request.address.clone()).await? {
                Some(name) => cached.push(name),
                None => missing.push(request),
            }
        }
        if missing.is_empty() {
            return Ok(cached);
        }

        let remote = match self.api.client.get_address_names(missing).await {
            Ok(names) => names,
            Err(_) => return Ok(cached),
        };
        self.store.save_address_names(remote.clone()).await?;
        cached.extend(remote);
        Ok(cached)
    }
}
