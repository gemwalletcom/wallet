use async_trait::async_trait;

use super::model::GemWalletConnectEvent;
use crate::services::error::GemServiceError;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemWalletConnectTransport: Send + Sync {
    async fn send(&self, message: String) -> Result<(), GemServiceError>;
}

#[uniffi::export(rust, foreign)]
pub trait GemWalletConnectListener: Send + Sync {
    fn on_event(&self, event: GemWalletConnectEvent);
}
