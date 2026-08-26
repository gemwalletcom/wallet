use std::sync::Arc;

use gem_api::GemDeviceApiClient as DeviceApiClient;

use crate::alien::{AlienClient, AlienProvider, new_alien_client};

#[derive(Debug, uniffi::Object)]
pub struct GemDeviceApiClient {
    pub(crate) client: DeviceApiClient<AlienClient>,
}

#[uniffi::export]
impl GemDeviceApiClient {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, base_url: String, device_private_key: Vec<u8>) -> Self {
        Self {
            client: DeviceApiClient::new(new_alien_client(base_url, provider), device_private_key),
        }
    }
}
