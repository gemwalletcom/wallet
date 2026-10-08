use async_trait::async_trait;
use primitives::{Currency, Device, Platform, PlatformStore};
use std::sync::Arc;

use super::GemDeviceService;
use super::platform::{GemDeviceInfo, GemDevicePlatform};
use crate::api::GemDeviceApiClient;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::subscription::GemSubscriptionService;
use crate::services::wallet_session::GemWalletSessionService;
use crate::testkit::TestAlienProvider;

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

impl GemDeviceService {
    pub fn mock(api: Arc<GemDeviceApiClient>, session: Arc<GemWalletSessionService>, preferences: Arc<GemPreferencesService>) -> Arc<Self> {
        Arc::new(Self::new(api.clone(), Arc::new(GemSubscriptionService::new(api, session)), Arc::new(MemoryDevicePlatform), preferences))
    }
}

pub fn registering_device_provider() -> TestAlienProvider {
    let device = serde_json::to_string(&Device::mock()).unwrap();
    TestAlienProvider::with_json_by_path(200, &[("devices/is-registered", "false"), ("devices/subscriptions", "[]"), ("/v3/devices", &device)])
}
