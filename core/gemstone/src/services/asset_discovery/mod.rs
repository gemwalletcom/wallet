pub mod rules;

use crate::services::error::GemServiceError;
use std::future::Future;
use std::sync::Arc;

use chrono::Utc;
use primitives::{AssetId, WalletId};

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

    pub async fn discover(&self, wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
        let (asset_ids, _, _) = futures::try_join!(
            self.discover_assets(wallet_id.clone()),
            self.complete(wallet_id.clone(), GemDiscoveryStep::Transactions, self.transactions.sync_wallet(wallet_id.clone(), None)),
            self.complete(wallet_id.clone(), GemDiscoveryStep::Nfts, async {
                self.nft.sync_wallet(wallet_id.clone()).await.map(|_| ())
            }),
        )?;
        Ok(asset_ids)
    }
}

impl GemAssetDiscoveryService {
    async fn discover_assets(&self, wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
        let Some(wallet) = self.wallet_store.get_wallet(wallet_id.clone())? else {
            return Ok(vec![]);
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
        Ok(asset_ids)
    }

    async fn complete<F>(&self, wallet_id: WalletId, step: GemDiscoveryStep, sync: F) -> Result<(), GemServiceError>
    where
        F: Future<Output = Result<(), GemServiceError>>,
    {
        if self.preferences.is_initial_load_completed(wallet_id.clone(), step)? {
            return Ok(());
        }
        sync.await?;
        self.preferences.set_initial_load_completed(wallet_id, step)
    }
}
