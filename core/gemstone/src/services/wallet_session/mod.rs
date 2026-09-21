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

    pub async fn require_current_wallet(&self) -> Result<Wallet, GemServiceError> {
        self.require_wallet(self.current_wallet_id()?).await
    }

    pub async fn require_wallet(&self, wallet_id: WalletId) -> Result<Wallet, GemServiceError> {
        self.wallets.get_wallet(wallet_id.clone()).await?.ok_or_else(|| GemServiceError::NotFound {
            msg: format!("wallet {} not found", wallet_id.id()),
        })
    }
}

impl GemWalletSessionService {
    pub fn current_wallet_id(&self) -> Result<WalletId, GemServiceError> {
        self.store.get_current_wallet_id()?.ok_or_else(|| GemServiceError::NotFound { msg: "no current wallet".to_string() })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use futures::executor::block_on;

    use super::testkit::MemoryWalletSessionStore;
    use super::*;
    use crate::services::wallet::testkit::MemoryWalletStore;

    fn service(wallet: Option<Wallet>, current: Option<WalletId>) -> GemWalletSessionService {
        GemWalletSessionService::new(
            Arc::new(MemoryWalletSessionStore { current: Mutex::new(current) }),
            Arc::new(MemoryWalletStore {
                wallets: Mutex::new(wallet.into_iter().collect()),
                ..Default::default()
            }),
        )
    }

    #[test]
    fn test_a_missing_current_wallet_is_an_error_the_apps_can_show() {
        block_on(async {
            let wallet = Wallet::mock();
            let current = service(Some(wallet.clone()), Some(wallet.id.clone()));

            assert_eq!(current.require_current_wallet().await.unwrap().id, wallet.id);
            assert!(current.get_current_wallet().await.unwrap().is_some());

            let empty = service(None, None);
            assert!(matches!(empty.require_current_wallet().await, Err(GemServiceError::NotFound { .. })));
            assert!(empty.get_current_wallet().await.unwrap().is_none(), "the optional read stays optional");
        })
    }

    #[test]
    fn test_a_wallet_that_is_gone_names_itself_in_the_error() {
        block_on(async {
            let wallet = Wallet::mock();
            let removed = service(None, Some(wallet.id.clone()));

            let error = removed.require_wallet(wallet.id.clone()).await.unwrap_err();

            assert!(error.to_string().contains(&wallet.id.id()), "{error}");
            assert!(matches!(removed.require_current_wallet().await, Err(GemServiceError::NotFound { .. })));
        })
    }
}
