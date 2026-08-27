use async_trait::async_trait;

use super::model::GemWalletConnectSignRequest;
use crate::services::error::GemServiceError;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemWalletConnectSigner: Send + Sync {
    async fn sign(&self, request: GemWalletConnectSignRequest) -> Result<String, GemServiceError>;
}
