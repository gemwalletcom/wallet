pub mod model;
#[cfg(test)]
pub(crate) mod testkit;

use std::future::Future;
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
use crate::services::support::GemSupportService;
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
    support: Arc<GemSupportService>,
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
        support: Arc<GemSupportService>,
    ) -> Self {
        Self {
            config,
            banners,
            assets,
            balance,
            wallet_configuration,
            wallet,
            device,
            support,
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
        match self.wallet.wallets().await {
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
        let default_assets = recorded(GemAppStartStep::SetupAssets, self.assets.ensure_default_assets()).await;
        let (banners, config_and_assets, device, support) = futures::join!(
            recorded(GemAppStartStep::SetupBanners, self.banners.setup()),
            self.sync_config_and_assets(),
            recorded(GemAppStartStep::SyncDevice, async { self.device.synchronize().await.map(|_| ()) }),
            recorded(GemAppStartStep::RecoverSupportMessages, self.support.recover_interrupted_messages()),
        );
        [default_assets, banners, config_and_assets, device, support].concat()
    }

    pub async fn setup_wallet(&self, wallet: Wallet) -> Vec<GemAppStartFailure> {
        let mut failures = Vec::new();
        record(&mut failures, GemAppStartStep::SetupAssets, self.assets.ensure_default_assets()).await;
        record(&mut failures, GemAppStartStep::SetupWalletBanners, self.banners.setup_wallet(wallet.clone())).await;
        record(&mut failures, GemAppStartStep::SetupWalletAssets, self.balance.setup_wallet(wallet.clone())).await;
        record(&mut failures, GemAppStartStep::SyncWalletConfiguration, self.wallet_configuration.sync(wallet.id)).await;
        failures
    }
}

impl GemAppStartService {
    async fn sync_config_and_assets(&self) -> Vec<GemAppStartFailure> {
        let mut failures = recorded(GemAppStartStep::UpdateConfig, async { self.config.update_config().await.map(|_| ()) }).await;
        record(&mut failures, GemAppStartStep::SyncAssets, self.sync_assets()).await;
        failures
    }

    async fn sync_assets(&self) -> Result<(), GemServiceError> {
        self.assets.sync_swappable_chains().await?;
        let config = self.config.get_config().await?;
        self.assets.sync_availability(config.versions).await
    }
}

async fn recorded<F>(step: GemAppStartStep, future: F) -> Vec<GemAppStartFailure>
where
    F: Future<Output = Result<(), GemServiceError>>,
{
    let mut failures = Vec::new();
    record(&mut failures, step, future).await;
    failures
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use primitives::AssetId;

    use chrono::Utc;
    use primitives::SupportMessageStatus;

    use super::testkit::AppStartTestkit;
    use super::*;
    use crate::services::support::{GemSupportStore, rules::pending_message};

    #[test]
    fn test_a_wallet_whose_keystore_cannot_be_read_does_not_stop_the_others() {
        block_on(async {
            let testkit = AppStartTestkit::new().await;
            testkit.wallets.lock_out(&testkit.first);

            let failures = testkit.service.setup_wallets().await;

            let chain_failures: Vec<&GemAppStartFailure> = failures.iter().filter(|failure| failure.step == GemAppStartStep::SetupChains).collect();
            assert_eq!(chain_failures.len(), 1, "{failures:?}");
            assert!(chain_failures[0].message.contains(&testkit.first.id.id()));
            assert!(!chain_failures[0].message.contains(&testkit.second.id.id()));
        })
    }

    #[test]
    fn test_the_app_start_fails_a_support_message_a_previous_run_left_sending() {
        block_on(async {
            let testkit = AppStartTestkit::new().await;
            testkit.support.save_messages(vec![pending_message("abandoned".into(), "hi".into(), vec![], Utc::now())]).await.unwrap();

            testkit.service.run().await;

            assert_eq!(testkit.support.statuses(), vec![("abandoned".to_string(), SupportMessageStatus::Failed)]);
        })
    }

    #[test]
    fn test_the_native_assets_a_banner_names_are_stored_before_the_app_start_writes_it() {
        block_on(async {
            let testkit = AppStartTestkit::new().await;

            let failures = testkit.service.run().await;

            let stored: Vec<AssetId> = testkit.assets.assets.lock().unwrap().iter().map(|basic| basic.asset.id.clone()).collect();
            let named: Vec<AssetId> = testkit.banners.writes.lock().unwrap().iter().flatten().filter_map(|key| key.asset_id.clone()).collect();

            assert!(!named.is_empty(), "the app start writes stake and perpetual banners");
            for asset_id in named {
                assert!(stored.contains(&asset_id), "banner names {asset_id}, which no asset row backs: {failures:?}");
            }
        })
    }

    #[test]
    fn test_every_wallet_is_set_up_when_the_keystores_open() {
        block_on(async {
            let testkit = AppStartTestkit::new().await;

            let failures = testkit.service.setup_wallets().await;

            assert!(!failures.iter().any(|failure| failure.step == GemAppStartStep::SetupChains), "{failures:?}");
        })
    }
}
