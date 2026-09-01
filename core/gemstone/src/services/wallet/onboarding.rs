use std::sync::Arc;

use primitives::{Chain, Wallet, WalletId};

use crate::services::avatar::GemAvatarService;
use crate::services::chain::GemChainService;
use crate::services::error::GemServiceError;
use crate::services::name::GemNameService;
use crate::services::wallet::{GemWalletImportResult, GemWalletImportType, GemWalletService, GemWalletSource};
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemOnboardingService {
    wallets: Arc<GemWalletService>,
    session: Arc<GemWalletSessionService>,
    avatars: Arc<GemAvatarService>,
    names: Arc<GemNameService>,
    chains: Arc<GemChainService>,
}

#[uniffi::export]
impl GemOnboardingService {
    #[uniffi::constructor]
    pub fn new(
        wallets: Arc<GemWalletService>,
        session: Arc<GemWalletSessionService>,
        avatars: Arc<GemAvatarService>,
        names: Arc<GemNameService>,
        chains: Arc<GemChainService>,
    ) -> Self {
        Self {
            wallets,
            session,
            avatars,
            names,
            chains,
        }
    }

    pub fn create_wallet(&self) -> Result<Vec<String>, GemServiceError> {
        self.wallets.create_wallet()
    }

    pub fn next_wallet_index(&self) -> Result<i32, GemServiceError> {
        self.wallets.next_wallet_index()
    }

    pub async fn import_wallet(&self, name: String, import: GemWalletImportType, source: GemWalletSource) -> Result<GemWalletImportResult, GemServiceError> {
        self.wallets.import_wallet(name, import, source).await
    }

    pub async fn rename(&self, wallet_id: WalletId, name: String) -> Result<(), GemServiceError> {
        self.wallets.rename(wallet_id, name).await
    }

    pub async fn setup_chains(&self, chains: Vec<Chain>) -> Result<Vec<Wallet>, GemServiceError> {
        self.wallets.setup_chains(chains).await
    }

    pub fn wallets(&self) -> Result<Vec<Wallet>, GemServiceError> {
        self.wallets.wallets()
    }

    pub fn avatars(&self) -> Arc<GemAvatarService> {
        self.avatars.clone()
    }

    pub fn names(&self) -> Arc<GemNameService> {
        self.names.clone()
    }

    pub fn chains(&self) -> Arc<GemChainService> {
        self.chains.clone()
    }

    pub fn session(&self) -> Arc<GemWalletSessionService> {
        self.session.clone()
    }
}
