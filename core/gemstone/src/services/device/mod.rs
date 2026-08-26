use std::sync::Arc;

use gem_client::ClientError;
use primitives::{Device, DeviceToken};

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemDeviceService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemDeviceService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_device(&self) -> Result<Option<Device>, GemApiError> {
        match self.api.client.get_device().await {
            Ok(device) => Ok(device),
            Err(ClientError::Http { status: 404, .. }) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub async fn add_device(&self, device: Device) -> Result<Device, GemApiError> {
        Ok(self.api.client.add_device(device).await?)
    }

    pub async fn update_device(&self, device: Device) -> Result<Device, GemApiError> {
        Ok(self.api.client.update_device(device).await?)
    }

    pub async fn is_registered(&self) -> Result<bool, GemApiError> {
        Ok(self.api.client.is_device_registered().await?)
    }

    pub async fn get_token(&self) -> Result<DeviceToken, GemApiError> {
        Ok(self.api.client.get_device_token().await?)
    }
}
