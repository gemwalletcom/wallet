use async_trait::async_trait;
use primitives::{Currency, Platform, PlatformStore};

use super::platform::{GemDeviceInfo, GemDevicePlatform};
use crate::services::error::GemServiceError;

pub struct MemoryDevicePlatform;

#[async_trait]
impl GemDevicePlatform for MemoryDevicePlatform {
    async fn device_id(&self) -> Result<String, GemServiceError> {
        Ok("device".to_string())
    }
    async fn device_info(&self) -> Result<GemDeviceInfo, GemServiceError> {
        Ok(GemDeviceInfo {
            platform: Platform::IOS,
            platform_store: PlatformStore::AppStore,
            os: "18".to_string(),
            model: "test".to_string(),
            version: "1.0".to_string(),
            locale_identifier: "en".to_string(),
        })
    }
    async fn push_token(&self) -> Result<String, GemServiceError> {
        Ok(String::new())
    }
    async fn is_push_enabled(&self) -> Result<bool, GemServiceError> {
        Ok(false)
    }
    async fn get_currency(&self) -> Result<Currency, GemServiceError> {
        Ok(Currency::USD)
    }
}
