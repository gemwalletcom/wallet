use std::sync::Arc;

use gem_api::GemApiClient as ApiClient;

use crate::alien::{AlienClient, AlienProvider, new_alien_client};

#[derive(Debug, uniffi::Object)]
pub struct GemApiClient {
    pub(crate) client: ApiClient<AlienClient>,
}

#[uniffi::export]
impl GemApiClient {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, base_url: String) -> Self {
        Self {
            client: ApiClient::new(new_alien_client(base_url, provider)),
        }
    }
}
