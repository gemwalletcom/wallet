use async_trait::async_trait;

use super::model::{GemWalletConnectMessageRequest, GemWalletConnectTransactionRequest};
use crate::services::error::GemServiceError;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemWalletConnectSigner: Send + Sync {
    async fn sign_message(&self, request: GemWalletConnectMessageRequest) -> Result<String, GemServiceError>;
    async fn sign_transaction(&self, request: GemWalletConnectTransactionRequest) -> Result<String, GemServiceError>;
}
