use async_trait::async_trait;

use crate::services::error::GemServiceError;
use primitives::WalletId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemKeystoreAuthentication {
    Biometrics,
    Passcode,
    None,
}

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemKeystorePassword: Send + Sync {
    async fn get_password(&self, create_if_missing: bool) -> Result<String, GemServiceError>;
    fn get_wallet_password(&self, wallet_id: WalletId) -> Result<Option<String>, GemServiceError>;
    fn delete_wallet_password(&self, wallet_id: WalletId) -> Result<(), GemServiceError>;
    fn authentication(&self) -> Result<GemKeystoreAuthentication, GemServiceError>;
}
