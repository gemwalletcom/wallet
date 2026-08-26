use async_trait::async_trait;
use primitives::Device;

use super::error::GemDeviceError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemDeviceStore: Send + Sync {
    async fn is_registered(&self) -> Result<bool, GemDeviceError>;
    async fn set_registered(&self, registered: bool) -> Result<(), GemDeviceError>;
    async fn get_subscriptions_version(&self) -> Result<i32, GemDeviceError>;
    async fn set_subscriptions_version(&self, version: i32) -> Result<(), GemDeviceError>;
    async fn get_pushed_device(&self) -> Result<Option<Device>, GemDeviceError>;
    async fn set_pushed_device(&self, device: Device) -> Result<(), GemDeviceError>;
    async fn get_pushed_subscriptions(&self) -> Result<Option<String>, GemDeviceError>;
    async fn set_pushed_subscriptions(&self, signature: String) -> Result<(), GemDeviceError>;
}
