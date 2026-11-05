use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

use gem_spark::{AccountKey, LightningError, LightningInvoice, LightningNetwork, LightningPayment, LightningWallet};
use tokio::runtime::{Builder, Runtime};

use crate::GemstoneError;
use crate::services::error::GemServiceError;

const RUNTIME_WORKER_THREADS: usize = 2;

static RUNTIME: LazyLock<Result<Runtime, String>> = LazyLock::new(|| {
    Builder::new_multi_thread()
        .worker_threads(RUNTIME_WORKER_THREADS)
        .thread_name("gem-lightning")
        .enable_all()
        .build()
        .map_err(|error| error.to_string())
});

#[derive(uniffi::Object)]
pub struct GemLightningWallet {
    wallet: Arc<LightningWallet>,
}

impl GemLightningWallet {
    pub(crate) fn new(wallet: Arc<LightningWallet>) -> Arc<Self> {
        Arc::new(Self { wallet })
    }
}

#[uniffi::export]
impl GemLightningWallet {
    pub async fn create_invoice(&self, amount_sats: Option<u64>, expiry_secs: Option<u32>) -> Result<LightningInvoice, GemstoneError> {
        let wallet = self.wallet.clone();
        Ok(run(async move { wallet.create_invoice(amount_sats, expiry_secs).await }).await?)
    }

    pub async fn payments(&self, limit: u64, offset: u64) -> Result<Vec<LightningPayment>, GemstoneError> {
        let wallet = self.wallet.clone();
        Ok(run(async move { wallet.payments(limit, offset).await }).await?)
    }

    pub async fn sync(&self) -> Result<(), GemstoneError> {
        let wallet = self.wallet.clone();
        Ok(run(async move { wallet.sync().await }).await?)
    }
}

pub(crate) async fn open_wallet(account: AccountKey, storage_dir: PathBuf) -> Result<LightningWallet, LightningError> {
    run(async move { LightningWallet::open(account, &storage_dir, LightningNetwork::Mainnet).await }).await
}

pub(crate) fn run<T: Send + 'static>(future: impl Future<Output = Result<T, LightningError>> + Send + 'static) -> impl Future<Output = Result<T, LightningError>> {
    let handle = RUNTIME.as_ref().map(|runtime| runtime.spawn(future)).map_err(|error| LightningError::Wallet(error.clone()));
    async move { handle?.await.map_err(|error| LightningError::Wallet(error.to_string()))? }
}

impl From<LightningError> for GemstoneError {
    fn from(error: LightningError) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<LightningError> for GemServiceError {
    fn from(error: LightningError) -> Self {
        match error {
            LightningError::Locked => Self::Cancelled,
            error => Self::core(error),
        }
    }
}
