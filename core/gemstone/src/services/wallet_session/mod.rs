pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use primitives::{Wallet, WalletId};

use crate::services::error::GemServiceError;
use crate::services::wallet::GemWalletStore;
use crate::services::wallet::rules::next_current_wallet;

pub use store::GemWalletSessionStore;

/// Holds `GemWalletStore` rather than `GemWalletService`, which owns it: the wallet service already
/// depends on this one, so the reverse edge would be a cycle. This is the narrow wallet query service.
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

    pub async fn ensure_current_wallet(&self) -> Result<Option<WalletId>, GemServiceError> {
        if self.get_current_wallet().await?.is_some() {
            return self.store.get_current_wallet_id();
        }
        let wallet_id = next_current_wallet(&self.wallets.get_wallets().await?);
        self.set_current_wallet_id(wallet_id.clone())?;
        Ok(wallet_id)
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
        service_with(wallet.into_iter().collect(), current)
    }

    fn service_with(wallets: Vec<Wallet>, current: Option<WalletId>) -> GemWalletSessionService {
        GemWalletSessionService::new(
            Arc::new(MemoryWalletSessionStore { current: Mutex::new(current) }),
            Arc::new(MemoryWalletStore {
                wallets: Mutex::new(wallets),
                ..Default::default()
            }),
        )
    }

    #[test]
    fn test_a_launch_without_a_current_wallet_recovers_one_from_the_stored_wallets() {
        block_on(async {
            let mut view = Wallet::mock_with_id(WalletId::View(primitives::Chain::Ethereum, "0xv".to_string()), &[primitives::Chain::Ethereum]);
            view.index = 0;
            let mut multicoin = Wallet::mock_with_id(WalletId::Multicoin("0x1".to_string()), &[primitives::Chain::Ethereum]);
            multicoin.index = 1;

            let missing = service_with(vec![view.clone(), multicoin.clone()], None);
            assert_eq!(missing.ensure_current_wallet().await.unwrap(), Some(multicoin.id.clone()));
            assert_eq!(missing.get_current_wallet_id().unwrap(), Some(multicoin.id.clone()));

            let stale = service_with(vec![view.clone()], Some(multicoin.id.clone()));
            assert_eq!(stale.ensure_current_wallet().await.unwrap(), Some(view.id.clone()), "a current id whose wallet is gone is replaced");

            let kept = service_with(vec![view.clone(), multicoin], Some(view.id.clone()));
            assert_eq!(kept.ensure_current_wallet().await.unwrap(), Some(view.id));

            let none = service_with(vec![], None);
            assert_eq!(none.ensure_current_wallet().await.unwrap(), None);
        })
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
