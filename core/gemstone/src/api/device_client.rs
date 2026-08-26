use std::sync::Arc;

use gem_api::GemDeviceApiClient as DeviceApiClient;

use crate::alien::{AlienError, AlienProvider, AlienProviderWrapper};

#[derive(Debug, uniffi::Object)]
pub struct GemDeviceApiClient {
    pub(crate) client: DeviceApiClient<AlienError>,
}

#[uniffi::export]
impl GemDeviceApiClient {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, base_url: String, device_private_key: Vec<u8>) -> Self {
        Self {
            client: DeviceApiClient::new(base_url, Arc::new(AlienProviderWrapper::new(provider)), device_private_key),
        }
    }
}
