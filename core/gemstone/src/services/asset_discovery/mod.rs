pub mod rules;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use std::future::Future;
use std::sync::Arc;

use primitives::WalletId;

pub use crate::services::wallet_preferences::GemDiscoveryStep;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::balance::GemBalanceService;
use crate::services::nft::GemNftService;
use crate::services::transactions::GemTransactionsService;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemAssetDiscoveryService {
    api: Arc<GemDeviceApiClient>,
    balance: Arc<GemBalanceService>,
    transactions: Arc<GemTransactionsService>,
    nft: Arc<GemNftService>,
    session: Arc<GemWalletSessionService>,
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
        session: Arc<GemWalletSessionService>,
        preferences: Arc<GemWalletPreferencesService>,
    ) -> Self {
        Self {
            api,
            balance,
            transactions,
            nft,
            session,
            preferences,
        }
    }
}

impl GemAssetDiscoveryService {
    pub async fn discover(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let (assets, transactions) = futures::join!(self.sync_assets(wallet_id.clone()), self.sync_transactions(wallet_id));
        assets?;
        transactions
    }

    pub async fn discover_nfts(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.run_initial_load(wallet_id.clone(), GemDiscoveryStep::Nfts, self.nft.sync_wallet(wallet_id)).await
    }

    async fn sync_assets(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let Some(wallet) = self.session.get_wallet(wallet_id.clone()).await? else {
            return Ok(());
        };
        let asset_ids = self.api.client.get_assets_list(wallet_id.id()).await.map_err(GemApiError::from)?;
        let asset_ids = rules::discoverable_asset_ids(asset_ids, &wallet.accounts);
        self.balance.enable_new_assets(wallet_id.clone(), asset_ids).await?;
        self.preferences.set_initial_load_completed(wallet_id, GemDiscoveryStep::Assets)?;
        Ok(())
    }

    async fn sync_transactions(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.run_initial_load(wallet_id.clone(), GemDiscoveryStep::Transactions, self.transactions.sync_wallet(wallet_id, None)).await
    }

    async fn run_initial_load<F, T>(&self, wallet_id: WalletId, step: GemDiscoveryStep, sync: F) -> Result<(), GemServiceError>
    where
        F: Future<Output = Result<T, GemServiceError>>,
    {
        if self.preferences.is_initial_load_completed(wallet_id.clone(), step) {
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
    use crate::services::wallet_preferences::GemDiscoveryStep;
    use crate::testkit::TestAlienProvider;

    #[test]
    fn test_discover_runs_the_assets_and_transactions_steps_and_reports_the_first_failure() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(503);

            assert!(testkit.discovery.discover(testkit.wallet_id.clone()).await.is_err());

            let paths = testkit.provider.requested_paths();
            for expected in ["devices/assets", "devices/transactions"] {
                assert!(paths.iter().any(|path| path.contains(expected)), "{expected} never ran: {paths:?}");
            }
            assert!(!paths.iter().any(|path| path.contains("devices/nft-assets")), "discover waited on NFTs: {paths:?}");
        })
    }

    #[test]
    fn test_nfts_load_once_and_are_not_loaded_again() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_response(TestAlienProvider::with_json_by_path(200, &[("devices/", "[]")]));

            testkit.discovery.discover_nfts(testkit.wallet_id.clone()).await.unwrap();
            testkit.discovery.discover_nfts(testkit.wallet_id.clone()).await.unwrap();

            let paths = testkit.provider.requested_paths();
            assert_eq!(paths.iter().filter(|path| path.contains("devices/nft-assets")).count(), 1, "{paths:?}");
            assert!(testkit.wallet_preferences.is_initial_load_completed(testkit.wallet_id.clone(), GemDiscoveryStep::Nfts));
        })
    }

    #[test]
    fn test_a_completed_step_is_not_loaded_again() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_response(TestAlienProvider::with_json_by_path(200, &[("devices/transactions", r#"{"transactions":[],"addressNames":[]}"#), ("devices/", "[]")]));

            testkit.discovery.discover(testkit.wallet_id.clone()).await.unwrap();
            let first = testkit.provider.requested_paths().len();
            testkit.discovery.discover(testkit.wallet_id.clone()).await.unwrap();
            let second = testkit.provider.requested_paths();

            assert_eq!(second.len(), first + 1, "a completed step loaded again: {second:?}");
            assert!(second.last().unwrap().contains("devices/assets"));
        })
    }

    #[test]
    fn test_discovery_enables_only_tokens_the_wallet_has_never_had() {
        use crate::services::assets::{GemAssetStore, rules::default_asset_basic};
        use crate::services::balance::GemAssetBalance;
        use primitives::{Asset, AssetId, AssetType, Chain};

        block_on(async {
            let known = AssetId::from_token(Chain::Ethereum, "0xknown");
            let unseen = AssetId::from_token(Chain::Ethereum, "0xunseen");
            let testkit = DiscoveryTestkit::with_response(TestAlienProvider::with_json_by_path(
                200,
                &[
                    ("devices/transactions", r#"{"transactions":[],"addressNames":[]}"#),
                    ("devices/assets", &format!("[\"{known}\",\"{unseen}\"]")),
                    ("devices/", "[]"),
                ],
            ));
            testkit.balances.balances.lock().unwrap().insert(testkit.wallet_id.clone(), vec![GemAssetBalance::zero(known.clone())]);
            testkit
                .asset_store
                .save_assets(vec![default_asset_basic(Asset::new(unseen.clone(), "Unseen".into(), "UNSEEN".into(), 18, AssetType::TOKEN))])
                .await
                .unwrap();

            testkit.discovery.discover(testkit.wallet_id.clone()).await.unwrap();

            let paths = testkit.provider.requested_paths();
            assert!(paths.iter().any(|path| path == "/v3/devices/assets"), "discovery must ask for the whole list, never a window: {paths:?}");
            assert_eq!(
                *testkit.balances.configuration_writes.lock().unwrap(),
                vec![(vec![unseen], crate::services::balance::rules::enabled_configuration(true))],
                "a token the wallet already has a row for, hidden or shown, must keep its own configuration"
            );
        })
    }
}
