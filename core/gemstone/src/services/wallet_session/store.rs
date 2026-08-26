use crate::services::error::GemServiceError;
use primitives::WalletId;

#[uniffi::export(with_foreign)]
pub trait GemWalletSessionStore: Send + Sync {
    fn get_current_wallet_id(&self) -> Result<Option<WalletId>, GemServiceError>;
    fn set_current_wallet_id(&self, wallet_id: Option<WalletId>) -> Result<(), GemServiceError>;
}
