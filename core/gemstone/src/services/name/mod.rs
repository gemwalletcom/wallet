pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{AddressName, Chain, ChainAddress};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::name::store::GemAddressNameWriter;
use crate::services::recipient::{GemRecipientValidation, rules as recipient_rules};

pub use model::{GemAddressNameUpdate, GemNameInputStep, GemNameRecordState};
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

    pub fn validate_recipient(&self, chain: Chain, input: String, state: GemNameRecordState) -> GemRecipientValidation {
        recipient_rules::validation(chain, &input, &state)
    }

    pub fn name_input_step(&self, state: GemNameRecordState, name: String, chain: Option<Chain>) -> GemNameInputStep {
        rules::name_input_step(&state, &name, chain)
    }

    pub fn resolved_state(&self, state: GemNameRecordState, name: String, chain: Chain, resolved: GemNameRecordState) -> GemNameRecordState {
        rules::resolved_state(&state, &name, chain, resolved)
    }

    pub async fn get_name_record(&self, name: String, chain: Chain) -> Result<GemNameRecordState, GemServiceError> {
        let record = self.api.client.get_name_record(name, chain.to_string()).await.map_err(GemApiError::from)?;
        Ok(rules::resolved(record))
    }
}

impl GemNameService {
    pub async fn address_name(&self, chain: Chain, address: String) -> Result<Option<AddressName>, GemServiceError> {
        self.store.get_address_name(chain, address).await
    }

    pub async fn save_names(&self, names: Vec<AddressName>) -> Result<(), GemServiceError> {
        self.store.save_names(names).await
    }

    pub async fn delete_names(&self, names: Vec<AddressName>) -> Result<(), GemServiceError> {
        self.store.delete_address_names(names).await
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
        self.store.save_names(remote.clone()).await?;
        cached.extend(remote);
        Ok(cached)
    }
}
