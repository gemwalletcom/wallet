use primitives::currency::Currency;
use primitives::{Platform, PlatformStore};

use crate::services::error::GemServiceError;
use async_trait::async_trait;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDeviceInfo {
    pub platform: Platform,
    pub platform_store: PlatformStore,
    pub os: String,
    pub model: String,
    pub version: String,
    pub locale_identifier: String,
}

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemDevicePlatform: Send + Sync {
    async fn device_id(&self) -> Result<String, GemServiceError>;
    async fn device_info(&self) -> Result<GemDeviceInfo, GemServiceError>;
    async fn push_token(&self) -> Result<String, GemServiceError>;
    async fn is_push_enabled(&self) -> Result<bool, GemServiceError>;
    async fn currency(&self) -> Result<Currency, GemServiceError>;
}
