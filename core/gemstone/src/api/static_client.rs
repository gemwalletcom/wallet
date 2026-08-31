use std::sync::Arc;

use gem_api::GemStaticApiClient as StaticApiClient;

use crate::alien::{AlienClient, AlienProvider, new_alien_client};

#[derive(Debug, uniffi::Object)]
pub struct GemStaticApiClient {
    pub(crate) client: StaticApiClient<AlienClient>,
}

#[uniffi::export]
impl GemStaticApiClient {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, base_url: String) -> Self {
        Self {
            client: StaticApiClient::new(new_alien_client(base_url, provider)),
        }
    }
}
