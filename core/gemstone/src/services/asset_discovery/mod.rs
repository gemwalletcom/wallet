pub mod rules;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use std::future::Future;
use std::sync::Arc;

use chrono::Utc;
use primitives::WalletId;

pub use crate::services::wallet_preferences::GemDiscoveryStep;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::balance::GemBalanceService;
use crate::services::nft::GemNftService;
use crate::services::transactions::GemTransactionsService;
use crate::services::wallet::GemWalletStore;
use crate::services::wallet_preferences::GemWalletPreferencesService;

#[derive(uniffi::Object)]
pub struct GemAssetDiscoveryService {
    api: Arc<GemDeviceApiClient>,
    balance: Arc<GemBalanceService>,
    transactions: Arc<GemTransactionsService>,
    nft: Arc<GemNftService>,
    wallet_store: Arc<dyn GemWalletStore>,
    preferences: Arc<GemWalletPreferencesService>,
}

#[uniffi::export]
impl GemAssetDiscoveryService {
    #[uniffi::constructor]
    pub fn new(
        api: Arc<GemDeviceApiClient>,
        balance: Arc<GemBalanceService>,
        transactions: Arc<GemTransactionsService>,
        nft: Arc<GemNftService>,
        wallet_store: Arc<dyn GemWalletStore>,
        preferences: Arc<GemWalletPreferencesService>,
    ) -> Self {
        Self {
            api,
            balance,
            transactions,
            nft,
            wallet_store,
            preferences,
        }
    }
}

impl GemAssetDiscoveryService {
    pub async fn discover(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let (assets, transactions, nfts) = futures::join!(self.sync_assets(wallet_id.clone()), self.sync_transactions(wallet_id.clone()), self.sync_nfts(wallet_id),);
        assets?;
        transactions?;
        nfts
    }

    async fn sync_assets(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let Some(wallet) = self.wallet_store.get_wallet(wallet_id.clone()).await? else {
            return Ok(());
        };
        let from_timestamp = self.preferences.get_assets_timestamp(wallet_id.clone());
        let timestamp = Utc::now().timestamp() as u64;
        let asset_ids = self.api.client.get_assets_list(wallet_id.id(), from_timestamp).await.map_err(GemApiError::from)?;
        let asset_ids = rules::discoverable_asset_ids(asset_ids, &wallet.accounts);
        if !asset_ids.is_empty() {
            self.balance.set_assets_enabled(wallet_id.clone(), asset_ids.clone(), true).await?;
        }
        self.preferences.set_assets_timestamp(wallet_id.clone(), timestamp)?;
        self.preferences.set_initial_load_completed(wallet_id, GemDiscoveryStep::Assets)?;
        Ok(())
    }

    async fn sync_transactions(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.run_initial_load(wallet_id.clone(), GemDiscoveryStep::Transactions, self.transactions.sync_wallet(wallet_id, None))
            .await
    }

    async fn sync_nfts(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.run_initial_load(wallet_id.clone(), GemDiscoveryStep::Nfts, self.nft.sync_wallet(wallet_id)).await
    }

    async fn run_initial_load<F, T>(&self, wallet_id: WalletId, step: GemDiscoveryStep, sync: F) -> Result<(), GemServiceError>
    where
        F: Future<Output = Result<T, GemServiceError>>,
    {
        if self.preferences.is_initial_load_completed(wallet_id.clone(), step)? {
            return Ok(());
        }
        sync.await?;
        self.preferences.set_initial_load_completed(wallet_id, step)
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::testkit::DiscoveryTestkit;
    use crate::testkit::TestAlienProvider;

    #[test]
    fn test_discover_runs_all_three_steps_and_reports_the_first_failure() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(503);

            assert!(testkit.discovery.discover(testkit.wallet_id.clone()).await.is_err());

            let paths = testkit.provider.requested_paths();
            for expected in ["devices/assets", "devices/transactions", "devices/nft_assets"] {
                assert!(paths.iter().any(|path| path.contains(expected)), "{expected} never ran: {paths:?}");
            }
        })
    }

    #[test]
    fn test_a_completed_step_is_not_loaded_again() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_response(TestAlienProvider::with_json_by_path(
                200,
                &[("devices/transactions", r#"{"transactions":[],"addressNames":[]}"#), ("devices/", "[]")],
            ));

            testkit.discovery.discover(testkit.wallet_id.clone()).await.unwrap();
            let first = testkit.provider.requested_paths().len();
            testkit.discovery.discover(testkit.wallet_id.clone()).await.unwrap();
            let second = testkit.provider.requested_paths();

            assert_eq!(second.len(), first + 1, "a completed step loaded again: {second:?}");
            assert!(second.last().unwrap().contains("devices/assets"));
        })
    }

    #[test]
    fn test_the_assets_step_records_its_timestamp_once_the_list_arrives() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_response(TestAlienProvider::with_json_by_path(
                200,
                &[("devices/transactions", r#"{"transactions":[],"addressNames":[]}"#), ("devices/", "[]")],
            ));

            assert_eq!(testkit.wallet_preferences.get_assets_timestamp(testkit.wallet_id.clone()), 0);

            testkit.discovery.discover(testkit.wallet_id.clone()).await.unwrap();

            assert!(testkit.wallet_preferences.get_assets_timestamp(testkit.wallet_id.clone()) > 0);
        })
    }
}
