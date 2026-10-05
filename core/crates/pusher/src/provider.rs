use async_trait::async_trait;
use gem_client::ClientError;
use push_notification::GorushNotification;

use crate::PushResult;

#[async_trait]
pub trait PushProvider: Send + Sync {
    async fn push_notifications(&self, notifications: Vec<GorushNotification>) -> Result<PushResult, ClientError>;
    async fn is_device_token_valid(&self, token: &str, platform: i32) -> Result<bool, ClientError>;
}
