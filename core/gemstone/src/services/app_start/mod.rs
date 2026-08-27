pub mod model;

use std::future::Future;
use std::sync::Arc;

use primitives::{Chain, Wallet};

pub use model::{GemAppStartFailure, GemAppStartStep};

use crate::services::assets::GemAssetsService;
use crate::services::banner::GemBannerService;
use crate::services::config::GemConfigService;
use crate::services::error::GemServiceError;
use crate::services::wallet::GemWalletService;
use crate::services::wallet_configuration::GemWalletConfigurationService;

#[derive(uniffi::Object)]
pub struct GemAppStartService {
    config: Arc<GemConfigService>,
    banners: Arc<GemBannerService>,
    assets: Arc<GemAssetsService>,
    wallet_configuration: Arc<GemWalletConfigurationService>,
    wallet: Arc<GemWalletService>,
}

#[uniffi::export]
impl GemAppStartService {
    #[uniffi::constructor]
    pub fn new(
        config: Arc<GemConfigService>,
        banners: Arc<GemBannerService>,
        assets: Arc<GemAssetsService>,
        wallet_configuration: Arc<GemWalletConfigurationService>,
        wallet: Arc<GemWalletService>,
    ) -> Self {
        Self {
            config,
            banners,
            assets,
            wallet_configuration,
            wallet,
        }
    }

    pub async fn setup_wallets(&self) -> Result<Vec<Wallet>, GemServiceError> {
        self.assets.sync_default_assets().await?;
        self.wallet.setup_chains(Chain::all()).await
    }

    pub async fn run(&self) -> Vec<GemAppStartFailure> {
        let mut failures = Vec::new();
        record(&mut failures, GemAppStartStep::UpdateConfig, async { self.config.update_config().await.map(|_| ()) }).await;
        record(&mut failures, GemAppStartStep::SetupBanners, self.banners.setup()).await;
        record(&mut failures, GemAppStartStep::SyncAssets, self.sync_assets()).await;
        failures
    }

    pub async fn setup_wallet(&self, wallet: Wallet) -> Vec<GemAppStartFailure> {
        let mut failures = Vec::new();
        record(&mut failures, GemAppStartStep::SetupWalletAssets, self.assets.setup_wallet(wallet.clone())).await;
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

async fn record<F>(failures: &mut Vec<GemAppStartFailure>, step: GemAppStartStep, future: F)
where
    F: Future<Output = Result<(), GemServiceError>>,
{
    if let Err(error) = future.await {
        failures.push(GemAppStartFailure { step, message: error.to_string() });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_collects_failures_and_continues() {
        let failures = futures::executor::block_on(async {
            let mut failures = Vec::new();
            record(&mut failures, GemAppStartStep::UpdateConfig, async {
                Err(GemServiceError::Status { msg: "offline".to_string() })
            })
            .await;
            record(&mut failures, GemAppStartStep::SetupBanners, async { Ok(()) }).await;
            record(&mut failures, GemAppStartStep::SyncAssets, async { Err(GemServiceError::Cancelled) }).await;
            failures
        });

        assert_eq!(
            failures,
            vec![
                GemAppStartFailure {
                    step: GemAppStartStep::UpdateConfig,
                    message: "offline".to_string()
                },
                GemAppStartFailure {
                    step: GemAppStartStep::SyncAssets,
                    message: "cancelled".to_string()
                },
            ]
        );
    }
}
