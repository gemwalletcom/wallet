use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{Wallet, WalletId};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemWalletStore: Send + Sync {
    async fn get_wallets(&self) -> Result<Vec<Wallet>, GemServiceError>;
    async fn get_wallet(&self, wallet_id: WalletId) -> Result<Option<Wallet>, GemServiceError>;
    async fn next_wallet_index(&self) -> Result<i32, GemServiceError>;
    async fn add_wallet(&self, wallet: Wallet) -> Result<(), GemServiceError>;
    async fn delete_wallet(&self, wallet_id: WalletId) -> Result<bool, GemServiceError>;
    async fn set_pinned(&self, wallet_id: WalletId, pinned: bool) -> Result<(), GemServiceError>;
    async fn rename(&self, wallet_id: WalletId, name: String) -> Result<(), GemServiceError>;
}
