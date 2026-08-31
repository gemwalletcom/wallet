pub mod store;

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

    pub fn get_current_wallet(&self) -> Result<Option<Wallet>, GemServiceError> {
        match self.store.get_current_wallet_id()? {
            Some(wallet_id) => self.wallets.get_wallet(wallet_id),
            None => Ok(None),
        }
    }

    pub fn get_wallets(&self) -> Result<Vec<Wallet>, GemServiceError> {
        self.wallets.get_wallets()
    }

    pub fn get_wallet(&self, wallet_id: WalletId) -> Result<Option<Wallet>, GemServiceError> {
        self.wallets.get_wallet(wallet_id)
    }
}
