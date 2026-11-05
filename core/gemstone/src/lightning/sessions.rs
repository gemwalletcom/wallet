use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, PoisonError, RwLock};

use async_trait::async_trait;
use futures::lock::Mutex as AsyncMutex;
use gem_spark::provider::{LightningAuthorization, LightningSessions};
use gem_spark::{LightningError, LightningInvoice, LightningPayment, LightningSendQuote, LightningWallet};

use super::wallet::run;

const PAYMENTS_LIMIT: u64 = 100;

static SESSIONS: LazyLock<Arc<LightningSessionRegistry>> = LazyLock::new(|| Arc::new(LightningSessionRegistry::default()));

pub(crate) fn sessions() -> Arc<LightningSessionRegistry> {
    SESSIONS.clone()
}

#[derive(Default)]
pub(crate) struct LightningSessionRegistry {
    wallets: RwLock<HashMap<String, Arc<LightningWallet>>>,
    opening: AsyncMutex<()>,
    generation: AtomicU64,
}

impl LightningSessionRegistry {
    pub(crate) async fn open<E, F, Fut>(&self, address: &str, open: F) -> Result<Arc<LightningWallet>, E>
    where
        E: From<LightningError>,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<LightningWallet, E>>,
    {
        let _opening = self.opening.lock().await;
        if let Ok(wallet) = self.wallet(address) {
            return Ok(wallet);
        }
        let generation = self.generation.load(Ordering::SeqCst);
        let wallet = Arc::new(open().await?);
        let mut wallets = self.wallets.write().unwrap_or_else(PoisonError::into_inner);
        if self.generation.load(Ordering::SeqCst) != generation {
            return Err(LightningError::Locked.into());
        }
        if wallet.address() != address {
            return Err(LightningError::AccountMismatch.into());
        }
        wallets.insert(address.to_string(), wallet.clone());
        Ok(wallet)
    }

    pub(crate) async fn close_and_remove<T>(&self, address: Option<&str>, remove: impl FnOnce() -> T) -> T {
        let _opening = self.opening.lock().await;
        let wallet = address.and_then(|address| self.wallets.write().unwrap_or_else(PoisonError::into_inner).remove(address));
        if let Some(wallet) = wallet {
            wallet.lock();
        }
        remove()
    }

    pub(crate) fn lock_all(&self) {
        let wallets: Vec<_> = {
            let mut wallets = self.wallets.write().unwrap_or_else(PoisonError::into_inner);
            self.generation.fetch_add(1, Ordering::SeqCst);
            wallets.drain().map(|(_, wallet)| wallet).collect()
        };
        wallets.iter().for_each(|wallet| wallet.lock());
    }

    pub(crate) fn wallet(&self, address: &str) -> Result<Arc<LightningWallet>, LightningError> {
        self.wallets.read().unwrap_or_else(PoisonError::into_inner).get(address).cloned().ok_or(LightningError::Locked)
    }
}

#[async_trait]
impl LightningSessions for LightningSessionRegistry {
    async fn balance(&self, address: &str) -> Result<u64, LightningError> {
        let wallet = self.wallet(address)?;
        run(async move { wallet.balance().await }).await
    }

    async fn payments(&self, address: &str) -> Result<Vec<LightningPayment>, LightningError> {
        let wallet = self.wallet(address)?;
        run(async move { wallet.payments(PAYMENTS_LIMIT, 0).await }).await
    }

    async fn payment(&self, address: &str, id: &str) -> Result<Option<LightningPayment>, LightningError> {
        let wallet = self.wallet(address)?;
        let id = id.to_string();
        run(async move { wallet.payment(&id).await }).await
    }

    async fn prepare_send(&self, address: &str, invoice: LightningInvoice, amount_sats: Option<u64>) -> Result<LightningSendQuote, LightningError> {
        let wallet = self.wallet(address)?;
        run(async move { wallet.prepare_send(invoice, amount_sats).await }).await
    }

    async fn send(&self, address: &str, quote: LightningSendQuote, authorization: LightningAuthorization) -> Result<String, LightningError> {
        let wallet = self.wallet(address)?;
        if !wallet.verify(&authorization.digest, &authorization.signature) {
            return Err(LightningError::Unauthorized);
        }
        run(async move { wallet.send(&quote).await }).await
    }
}
