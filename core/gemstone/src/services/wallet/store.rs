use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{Wallet, WalletId};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemWalletStore: Send + Sync {
    fn get_wallets(&self) -> Result<Vec<Wallet>, GemServiceError>;
    fn get_wallet(&self, wallet_id: WalletId) -> Result<Option<Wallet>, GemServiceError>;
    async fn add_wallet(&self, wallet: Wallet) -> Result<(), GemServiceError>;
    async fn delete_wallet(&self, wallet_id: WalletId) -> Result<bool, GemServiceError>;
    async fn set_pinned(&self, wallet_id: WalletId, pinned: bool) -> Result<(), GemServiceError>;
    async fn rename(&self, wallet_id: WalletId, name: String) -> Result<(), GemServiceError>;
    async fn set_image_url(&self, wallet_id: WalletId, image_url: Option<String>) -> Result<(), GemServiceError>;
}
