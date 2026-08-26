use std::sync::Arc;

use async_trait::async_trait;
use gem_api::{GemDeviceApiClient as DeviceApiClient, WalletRequestPreflight};
use gem_client::ClientError;

use crate::alien::{AlienError, AlienProvider, AlienProviderWrapper};
use crate::api::GemApiError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemWalletRequestPreflight: Send + Sync + std::fmt::Debug {
    async fn prepare(&self) -> Result<(), GemApiError>;
}

#[derive(Debug)]
struct PreflightWrapper {
    preflight: Arc<dyn GemWalletRequestPreflight>,
}

#[async_trait]
impl WalletRequestPreflight for PreflightWrapper {
    async fn prepare(&self) -> Result<(), ClientError> {
        self.preflight.prepare().await.map_err(|error| ClientError::Network(error.to_string()))
    }
}

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

    #[uniffi::constructor]
    pub fn with_preflight(provider: Arc<dyn AlienProvider>, base_url: String, device_private_key: Vec<u8>, preflight: Arc<dyn GemWalletRequestPreflight>) -> Self {
        Self {
            client: DeviceApiClient::new(base_url, Arc::new(AlienProviderWrapper::new(provider)), device_private_key).with_preflight(Arc::new(PreflightWrapper { preflight })),
        }
    }
}
