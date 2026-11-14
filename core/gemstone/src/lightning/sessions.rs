use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, LazyLock, PoisonError, RwLock};

use async_trait::async_trait;
use futures::lock::Mutex as AsyncMutex;
use gem_spark::chain_signer::LightningAuthorization;
use gem_spark::provider::LightningSessions;
use gem_spark::{LightningError, LightningInvoice, LightningPayment, LightningPaymentStatus, LightningSendQuote, LightningWallet};

use super::runtime::run;
use crate::services::error::GemServiceError;

static SESSIONS: LazyLock<Arc<LightningSessionRegistry>> = LazyLock::new(|| Arc::new(LightningSessionRegistry::default()));

pub(crate) fn sessions() -> Arc<LightningSessionRegistry> {
    SESSIONS.clone()
}

#[derive(Default)]
pub(crate) struct LightningSessionRegistry {
    wallets: RwLock<HashMap<String, Arc<LightningWallet>>>,
    opening: AsyncMutex<()>,
}

impl LightningSessionRegistry {
    pub(crate) async fn open<F, Fut>(&self, address: &str, open: F) -> Result<(Arc<LightningWallet>, bool), GemServiceError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<LightningWallet, GemServiceError>>,
    {
        let _opening = self.opening.lock().await;
        if let Ok(wallet) = self.wallet(address) {
            return Ok((wallet, false));
        }
        let wallet = Arc::new(open().await?);
        if wallet.address() != address {
            return Err(LightningError::AccountMismatch.into());
        }
        self.wallets.write().unwrap_or_else(PoisonError::into_inner).insert(address.to_string(), wallet.clone());
        Ok((wallet, true))
    }

    pub(crate) async fn close_and_remove(&self, address: Option<&str>, remove: impl FnOnce() -> Result<(), GemServiceError>) -> Result<(), GemServiceError> {
        let _opening = self.opening.lock().await;
        if let Some(address) = address {
            self.wallets.write().unwrap_or_else(PoisonError::into_inner).remove(address);
        }
        remove()
    }

    fn wallet(&self, address: &str) -> Result<Arc<LightningWallet>, LightningError> {
        self.wallets.read().unwrap_or_else(PoisonError::into_inner).get(address).cloned().ok_or(LightningError::Locked)
    }

    pub(crate) async fn payments(&self, address: &str) -> Result<Vec<LightningPayment>, LightningError> {
        self.with_wallet(address, |wallet| async move { wallet.payments().await }).await
    }

    async fn with_wallet<T, F, Fut>(&self, address: &str, call: F) -> Result<T, LightningError>
    where
        T: Send + 'static,
        F: FnOnce(Arc<LightningWallet>) -> Fut,
        Fut: Future<Output = Result<T, LightningError>> + Send + 'static,
    {
        run(call(self.wallet(address)?)).await
    }
}

#[async_trait]
impl LightningSessions for LightningSessionRegistry {
    async fn balance(&self, address: &str) -> Result<u64, LightningError> {
        self.with_wallet(address, |wallet| async move { wallet.balance().await }).await
    }

    async fn payment_status(&self, address: &str, id: &str) -> Result<LightningPaymentStatus, LightningError> {
        let id = id.to_string();
        self.with_wallet(address, |wallet| async move { wallet.payment_status(&id).await }).await
    }

    async fn send_fee(&self, address: &str, invoice: LightningInvoice, amount_sats: u64) -> Result<u64, LightningError> {
        self.with_wallet(address, |wallet| async move { wallet.send_fee(&invoice, amount_sats).await }).await
    }

    async fn send(&self, address: &str, quote: LightningSendQuote, authorization: LightningAuthorization) -> Result<String, LightningError> {
        let wallet = self.wallet(address)?;
        if !wallet.verify(&authorization.digest, &authorization.signature) {
            return Err(LightningError::Unauthorized);
        }
        run(async move { wallet.send(&quote).await }).await
    }
}
