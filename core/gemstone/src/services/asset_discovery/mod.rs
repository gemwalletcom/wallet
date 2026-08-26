pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use chrono::Utc;
use primitives::currency::Currency;
use primitives::{AssetId, WalletId};

pub use store::GemAssetDiscoveryStore;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::balance::GemBalanceService;
use crate::services::subscription::GemWalletStore;

#[derive(uniffi::Object)]
pub struct GemAssetDiscoveryService {
    api: Arc<GemDeviceApiClient>,
    balance: Arc<GemBalanceService>,
    wallet_store: Arc<dyn GemWalletStore>,
    store: Arc<dyn GemAssetDiscoveryStore>,
}

#[uniffi::export]
impl GemAssetDiscoveryService {
    #[uniffi::constructor]
    pub fn new(
        api: Arc<GemDeviceApiClient>,
        balance: Arc<GemBalanceService>,
        wallet_store: Arc<dyn GemWalletStore>,
        store: Arc<dyn GemAssetDiscoveryStore>,
    ) -> Self {
        Self {
            api,
            balance,
            wallet_store,
            store,
        }
    }

    pub async fn discover(&self, wallet_id: WalletId, currency: Currency) -> Result<Vec<AssetId>, GemServiceError> {
        let Some(wallet) = self.wallet_store.get_wallet(wallet_id.clone()).await? else {
            return Ok(vec![]);
        };
        let from_timestamp = self.store.get_assets_timestamp(wallet_id.clone()).await?;
        let timestamp = Utc::now().timestamp() as u64;
        let asset_ids = self.api.client.get_assets_list(wallet_id.id(), from_timestamp).await.map_err(GemApiError::from)?;
        let asset_ids = rules::discoverable_asset_ids(asset_ids, &wallet.accounts);
        if !asset_ids.is_empty() {
            self.balance.enable_assets(wallet_id.clone(), asset_ids.clone(), true, currency).await?;
        }
        self.store.set_assets_timestamp(wallet_id, timestamp).await?;
        Ok(asset_ids)
    }
}
