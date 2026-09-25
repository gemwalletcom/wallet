pub mod model;
mod rules;

use std::sync::Arc;

use primitives::{Chain, ChainAddress};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::models::state::GemLoad;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
pub use model::GemAddressDetails;

#[derive(uniffi::Object)]
pub struct GemAddressDetailsService {
    api: Arc<GemDeviceApiClient>,
    explorer: Arc<GemExplorerService>,
}

#[uniffi::export]
impl GemAddressDetailsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, explorer: Arc<GemExplorerService>) -> Self {
        Self { api, explorer }
    }

    pub fn details(&self, chain: Chain, address: String) -> GemAddressDetails {
        let link = self.explorer.get_address_url(chain, address.clone());
        rules::details(chain, address, link, GemLoad::loading())
    }

    pub async fn refresh(&self, details: GemAddressDetails) -> GemAddressDetails {
        let request = ChainAddress::new(details.chain, details.address.clone());
        let result = self.api.client.get_address_details(request).await.map_err(|error| GemServiceError::from(GemApiError::from(error)));
        rules::refreshed(details, result)
    }
}
