use crate::services::error::GemServiceError;
use primitives::WalletId;

#[uniffi::export(with_foreign)]
pub trait GemKeystorePassword: Send + Sync {
    fn get_password(&self, wallet_id: WalletId, create_if_missing: bool) -> Result<Vec<u8>, GemServiceError>;
}
