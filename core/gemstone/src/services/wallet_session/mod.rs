pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use primitives::{Wallet, WalletId};

use crate::services::error::GemServiceError;
use crate::services::wallet::GemWalletStore;

pub use store::GemWalletSessionStore;

#[derive(uniffi::Object)]
pub struct GemWalletSessionService {
    store: Arc<dyn GemWalletSessionStore>,
    wallets: Arc<dyn GemWalletStore>,
}

#[uniffi::export]
impl GemWalletSessionService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemWalletSessionStore>, wallets: Arc<dyn GemWalletStore>) -> Self {
        Self { store, wallets }
    }

    pub fn get_current_wallet_id(&self) -> Result<Option<WalletId>, GemServiceError> {
        self.store.get_current_wallet_id()
    }

    pub fn set_current_wallet_id(&self, wallet_id: Option<WalletId>) -> Result<(), GemServiceError> {
        if self.store.get_current_wallet_id()? == wallet_id {
            return Ok(());
        }
        self.store.set_current_wallet_id(wallet_id)
    }

    pub async fn get_current_wallet(&self) -> Result<Option<Wallet>, GemServiceError> {
        match self.store.get_current_wallet_id()? {
            Some(wallet_id) => self.wallets.get_wallet(wallet_id).await,
            None => Ok(None),
        }
    }

    pub fn shows_rewards(&self, wallets: Vec<Wallet>) -> bool {
        rules::shows_rewards(&wallets)
    }

    pub async fn get_wallets(&self) -> Result<Vec<Wallet>, GemServiceError> {
        self.wallets.get_wallets().await
    }

    pub async fn get_wallet(&self, wallet_id: WalletId) -> Result<Option<Wallet>, GemServiceError> {
        self.wallets.get_wallet(wallet_id).await
    }
}

impl GemWalletSessionService {
    pub fn current_wallet_id(&self) -> Result<WalletId, GemServiceError> {
        self.store.get_current_wallet_id()?.ok_or_else(|| GemServiceError::NotFound {
            msg: "no current wallet".to_string(),
        })
    }

    pub async fn current_wallet(&self) -> Result<Wallet, GemServiceError> {
        let wallet_id = self.current_wallet_id()?;
        self.wallets.get_wallet(wallet_id.clone()).await?.ok_or_else(|| GemServiceError::NotFound {
            msg: format!("wallet {} not found", wallet_id.id()),
        })
    }
}
