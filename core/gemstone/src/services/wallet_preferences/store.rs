use crate::services::error::GemServiceError;
use primitives::WalletId;

#[uniffi::export(rust, foreign)]
pub trait GemWalletPreferencesStore: Send + Sync {
    fn get(&self, wallet_id: WalletId, key: String) -> Option<String>;
    fn set(&self, wallet_id: WalletId, key: String, value: String) -> Result<(), GemServiceError>;
    fn clear(&self, wallet_id: WalletId) -> Result<(), GemServiceError>;
}
