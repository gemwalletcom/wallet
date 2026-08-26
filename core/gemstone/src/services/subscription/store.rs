use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::{Wallet, WalletId};

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemWalletStore: Send + Sync {
    async fn get_wallets(&self) -> Result<Vec<Wallet>, GemServiceError>;
    async fn get_wallet(&self, wallet_id: WalletId) -> Result<Option<Wallet>, GemServiceError>;
}
