pub mod model;

use std::sync::Arc;

use primitives::{Chain, Wallet};

pub use model::{GemAppStartFailure, GemAppStartStep};

use crate::services::assets::GemAssetsService;
use crate::services::balance::GemBalanceService;
use crate::services::banner::GemBannerService;
use crate::services::config::GemConfigService;
use crate::services::device::GemDeviceService;
use crate::services::error::GemServiceError;
use crate::services::failures::{StepFailure, record};
use crate::services::wallet::GemWalletService;
use crate::services::wallet_configuration::GemWalletConfigurationService;

#[derive(uniffi::Object)]
pub struct GemAppStartService {
    config: Arc<GemConfigService>,
    banners: Arc<GemBannerService>,
    assets: Arc<GemAssetsService>,
    balance: Arc<GemBalanceService>,
    wallet_configuration: Arc<GemWalletConfigurationService>,
    wallet: Arc<GemWalletService>,
    device: Arc<GemDeviceService>,
}

#[uniffi::export]
impl GemAppStartService {
    #[uniffi::constructor]
    pub fn new(
        config: Arc<GemConfigService>,
        banners: Arc<GemBannerService>,
        assets: Arc<GemAssetsService>,
        balance: Arc<GemBalanceService>,
        wallet_configuration: Arc<GemWalletConfigurationService>,
        wallet: Arc<GemWalletService>,
        device: Arc<GemDeviceService>,
    ) -> Self {
        Self {
            config,
            banners,
            assets,
            balance,
            wallet_configuration,
            wallet,
            device,
        }
    }

    pub async fn setup_wallets(&self) -> Vec<GemAppStartFailure> {
        let mut failures = Vec::new();
        record(&mut failures, GemAppStartStep::SetupWalletAssets, self.assets.sync_default_assets()).await;
        match self.wallet.setup_chains_outcome(Chain::all()).await {
            Ok(outcome) => {
                for (wallet_id, error) in outcome.failures {
                    failures.push(GemAppStartFailure::new(GemAppStartStep::SetupChains, format!("wallet {}: {error}", wallet_id.id())));
                }
            }
            Err(error) => failures.push(GemAppStartFailure::new(GemAppStartStep::SetupChains, error.to_string())),
        }
        match self.wallet.wallets() {
            Ok(wallets) => {
                for wallet in wallets {
                    let wallet_id = wallet.id.clone();
                    if let Err(error) = self.balance.setup_wallet(wallet).await {
                        failures.push(GemAppStartFailure::new(GemAppStartStep::SetupWalletAssets, format!("wallet {}: {error}", wallet_id.id())));
                    }
                }
            }
            Err(error) => failures.push(GemAppStartFailure::new(GemAppStartStep::SetupWalletAssets, error.to_string())),
        }
        failures
    }

    pub async fn run(&self) -> Vec<GemAppStartFailure> {
        let mut failures = Vec::new();
        record(&mut failures, GemAppStartStep::UpdateConfig, async { self.config.update_config().await.map(|_| ()) }).await;
        record(&mut failures, GemAppStartStep::SetupBanners, self.banners.setup()).await;
        record(&mut failures, GemAppStartStep::SyncAssets, self.sync_assets()).await;
        record(&mut failures, GemAppStartStep::SyncDevice, async { self.device.synchronize().await.map(|_| ()) }).await;
        failures
    }

    pub async fn setup_wallet(&self, wallet: Wallet) -> Vec<GemAppStartFailure> {
        let mut failures = Vec::new();
        record(&mut failures, GemAppStartStep::SetupWalletAssets, async {
            self.assets.sync_default_assets().await?;
            self.balance.setup_wallet(wallet.clone()).await
        })
        .await;
        record(&mut failures, GemAppStartStep::SetupWalletBanners, self.banners.setup_wallet(wallet.clone())).await;
        record(&mut failures, GemAppStartStep::SyncWalletConfiguration, self.wallet_configuration.sync(wallet.id)).await;
        failures
    }
}

impl GemAppStartService {
    async fn sync_assets(&self) -> Result<(), GemServiceError> {
        self.assets.sync_swappable_chains().await?;
        let config = self.config.get_config().await?;
        self.assets.sync_availability(config.versions).await
    }
}
