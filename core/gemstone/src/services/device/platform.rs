use primitives::currency::Currency;
use primitives::{DeviceLocale, Platform, PlatformStore};

use crate::services::error::GemServiceError;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemDeviceInfo {
    pub platform: Platform,
    pub platform_store: PlatformStore,
    pub os: String,
    pub model: String,
    pub version: String,
    pub locale: DeviceLocale,
}

#[uniffi::export(rust, foreign)]
pub trait GemDevicePlatform: Send + Sync {
    fn device_id(&self) -> Result<String, GemServiceError>;
    fn device_info(&self) -> Result<GemDeviceInfo, GemServiceError>;
    fn push_token(&self) -> Result<String, GemServiceError>;
    fn is_push_enabled(&self) -> Result<bool, GemServiceError>;
    fn currency(&self) -> Result<Currency, GemServiceError>;
}
