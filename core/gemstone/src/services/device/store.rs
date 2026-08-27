use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::Device;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemDeviceStore: Send + Sync {
    async fn is_registered(&self) -> Result<bool, GemServiceError>;
    async fn set_registered(&self, registered: bool) -> Result<(), GemServiceError>;
    async fn get_subscriptions_version(&self) -> Result<i32, GemServiceError>;
    async fn set_subscriptions_version(&self, version: i32) -> Result<(), GemServiceError>;
    async fn get_pushed_device(&self) -> Result<Option<Device>, GemServiceError>;
    async fn set_pushed_device(&self, device: Device) -> Result<(), GemServiceError>;
    async fn get_pushed_subscriptions(&self) -> Result<Option<String>, GemServiceError>;
    async fn set_pushed_subscriptions(&self, signature: String) -> Result<(), GemServiceError>;
}
