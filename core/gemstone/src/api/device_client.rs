use std::sync::Arc;

use gem_api::GemDeviceApiClient as DeviceApiClient;

use crate::services::device::{DeviceSyncPreflight, GemDeviceKeyService, GemDeviceService};

use crate::alien::{AlienError, AlienProvider, AlienProviderWrapper};
use crate::config::public::API_URL;

#[derive(Debug, uniffi::Object)]
pub struct GemDeviceApiClient {
    pub(crate) client: DeviceApiClient<AlienError>,
}

#[uniffi::export]
impl GemDeviceApiClient {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, device_key: Arc<GemDeviceKeyService>) -> Self {
        Self {
            client: DeviceApiClient::new(API_URL.to_string(), Arc::new(AlienProviderWrapper::new(provider)), device_key),
        }
    }
}

#[uniffi::export]
impl GemDeviceApiClient {
    pub fn set_device_sync_preflight(&self, device: Arc<GemDeviceService>) {
        self.client.set_preflight(Arc::new(DeviceSyncPreflight { service: Arc::downgrade(&device) }));
    }
}
