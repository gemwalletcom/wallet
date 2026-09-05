pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::name::NameRecord;
use primitives::{AddressName, Chain, ChainAddress};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::recipient::{GemRecipientError, GemRecipientValidation, rules as recipient_rules};
use crate::services::transfer::GemRecipient;

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

    pub fn validate_recipient(&self, chain: Chain, input: String, name_record: Option<NameRecord>) -> GemRecipientValidation {
        recipient_rules::validation(chain, &input, name_record.as_ref())
    }

    pub fn recipient(
        &self,
        chain: Chain,
        input: String,
        name_record: Option<NameRecord>,
        memo: Option<String>,
        references: Vec<String>,
    ) -> Result<GemRecipient, GemRecipientError> {
        recipient_rules::recipient(chain, &input, name_record.as_ref(), memo, references)
    }

    pub fn is_name_supported(&self, name: String) -> bool {
        rules::is_name_supported(&name)
    }

    pub fn name_record_debounce_milliseconds(&self) -> u64 {
        rules::name_record_debounce_milliseconds()
    }

    pub async fn get_name_record(&self, name: String, chain: Chain) -> Result<Option<NameRecord>, GemServiceError> {
        Ok(self.api.client.get_name_record(name, chain.to_string()).await.map_err(GemApiError::from)?)
    }

    pub async fn address_name(&self, chain: Chain, address: String) -> Result<Option<AddressName>, GemServiceError> {
        self.store.get_address_name(chain, address).await
    }
}

impl GemNameService {
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
