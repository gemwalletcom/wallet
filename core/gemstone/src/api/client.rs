use std::sync::Arc;

use gem_api::GemApiClient as ApiClient;

use crate::alien::{AlienClient, AlienProvider, coalescing_provider, new_alien_client};
use crate::config::public::API_URL;

#[derive(Debug, uniffi::Object)]
pub struct GemApiClient {
    pub(crate) client: ApiClient<AlienClient>,
}

#[uniffi::export]
impl GemApiClient {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            client: ApiClient::new(new_alien_client(API_URL.to_string(), coalescing_provider(provider))),
        }
    }
}
