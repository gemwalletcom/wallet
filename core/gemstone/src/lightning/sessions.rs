use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::sync::{Arc, LazyLock, PoisonError, RwLock};

use async_trait::async_trait;
use futures::lock::Mutex as AsyncMutex;
use gem_spark::chain_signer::SignedLightningTransfer;
use gem_spark::provider::LightningSessions;
use gem_spark::{AccountKey, LightningError, LightningInvoice, LightningPayment, LightningPaymentStatus, LightningWallet};
use primitives::Chain;

use super::runtime::run;
use super::store::WalletLightningStore;
use crate::keystore::keystore_id_for_wallet;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemSecureStore;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_session::GemWalletSessionService;

pub(crate) static SESSIONS: LazyLock<Arc<LightningSessionRegistry>> = LazyLock::new(|| Arc::new(LightningSessionRegistry::default()));

pub(crate) fn account_key_name(keystore_id: &str) -> String {
    format!("lightning_account_{keystore_id}")
}

pub(crate) struct LightningAccounts {
    pub(crate) secure: Arc<dyn GemSecureStore>,
    pub(crate) preferences: Arc<GemWalletPreferencesService>,
    pub(crate) session: Arc<GemWalletSessionService>,
}

impl LightningAccounts {
    async fn open(&self, address: &str) -> Result<LightningWallet, GemServiceError> {
        let wallet = self
            .session
            .get_wallets()
            .await?
            .into_iter()
            .find(|wallet| wallet.account(Chain::Lightning).is_some_and(|account| account.address == address))
            .ok_or(GemServiceError::Cancelled)?;
        let key = self.secure.get(account_key_name(&keystore_id_for_wallet(wallet.id.id())))?.ok_or(GemServiceError::Cancelled)?;
        let account = AccountKey::decode(&key)?;
        let store = Arc::new(WalletLightningStore::new(self.preferences.clone(), wallet.id));
        Ok(run(async move { LightningWallet::open(account, store).await }).await?)
    }
}

#[derive(Default)]
pub(crate) struct LightningSessionRegistry {
    wallets: RwLock<HashMap<String, Arc<LightningWallet>>>,
    opening: AsyncMutex<()>,
    accounts: RwLock<Option<Arc<LightningAccounts>>>,
}

impl LightningSessionRegistry {
    pub(crate) fn set_accounts(&self, accounts: LightningAccounts) {
        *self.accounts.write().unwrap_or_else(PoisonError::into_inner) = Some(Arc::new(accounts));
    }

    pub(crate) async fn open(&self, address: &str) -> Result<(Arc<LightningWallet>, bool), GemServiceError> {
        let _opening = self.opening.lock().await;
        if let Some(wallet) = self.open_wallet(address) {
            return Ok((wallet, false));
        }
        let accounts = self.accounts.read().unwrap_or_else(PoisonError::into_inner).clone().ok_or(GemServiceError::Cancelled)?;
        let wallet = Arc::new(accounts.open(address).await?);
        if wallet.address() != address {
            return Err(LightningError::AccountMismatch.into());
        }
        self.wallets.write().unwrap_or_else(PoisonError::into_inner).insert(address.to_string(), wallet.clone());
        Ok((wallet, true))
    }

    pub(crate) async fn wallet(&self, address: &str) -> Result<Arc<LightningWallet>, GemServiceError> {
        match self.open_wallet(address) {
            Some(wallet) => Ok(wallet),
            None => Ok(self.open(address).await?.0),
        }
    }

    pub(crate) async fn close_and_remove(&self, address: Option<&str>, remove: impl FnOnce() -> Result<(), GemServiceError>) -> Result<(), GemServiceError> {
        let _opening = self.opening.lock().await;
        if let Some(address) = address {
            self.wallets.write().unwrap_or_else(PoisonError::into_inner).remove(address);
        }
        remove()
    }

    fn open_wallet(&self, address: &str) -> Option<Arc<LightningWallet>> {
        self.wallets.read().unwrap_or_else(PoisonError::into_inner).get(address).cloned()
    }

    pub(crate) async fn payments(&self, address: &str) -> Result<Vec<LightningPayment>, GemServiceError> {
        self.with_wallet(address, |wallet| async move { wallet.payments().await }).await
    }

    async fn with_wallet<T, F, Fut>(&self, address: &str, call: F) -> Result<T, GemServiceError>
    where
        T: Send + 'static,
        F: FnOnce(Arc<LightningWallet>) -> Fut,
        Fut: Future<Output = Result<T, LightningError>> + Send + 'static,
    {
        Ok(run(call(self.wallet(address).await?)).await?)
    }
}

#[async_trait]
impl LightningSessions for LightningSessionRegistry {
    async fn balance(&self, address: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
        Ok(self.with_wallet(address, |wallet| async move { wallet.balance().await }).await?)
    }

    async fn payment_status(&self, address: &str, id: &str) -> Result<LightningPaymentStatus, Box<dyn Error + Send + Sync>> {
        let id = id.to_string();
        Ok(self.with_wallet(address, |wallet| async move { wallet.payment_status(&id).await }).await?)
    }

    async fn send_fee(&self, address: &str, invoice: LightningInvoice, amount_sats: u64) -> Result<u64, Box<dyn Error + Send + Sync>> {
        Ok(self.with_wallet(address, |wallet| async move { wallet.send_fee(&invoice, amount_sats).await }).await?)
    }

    async fn send(&self, signed: SignedLightningTransfer) -> Result<String, Box<dyn Error + Send + Sync>> {
        let wallet = self.wallet(&signed.transfer.address).await?;
        if !wallet.verify(&signed.digest(), &signed.signature) {
            return Err(LightningError::Unauthorized.into());
        }
        Ok(run(async move { wallet.send(&signed.transfer).await }).await?)
    }
}
