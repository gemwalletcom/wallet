use async_trait::async_trait;
use primitives::Wallet;
use primitives::WalletId;

use super::error::GemSubscriptionError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemWalletStore: Send + Sync {
    async fn get_wallets(&self) -> Result<Vec<Wallet>, GemSubscriptionError>;
    async fn get_wallet(&self, wallet_id: WalletId) -> Result<Option<Wallet>, GemSubscriptionError>;
}
