pub mod error;
pub mod rules;
pub mod store;

use std::sync::Arc;

use chrono::Utc;
use primitives::{AssetId, WalletId};

pub use error::GemAssetDiscoveryError;
pub use store::GemAssetDiscoveryStore;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::assets::GemAssetsService;
use crate::services::subscription::GemWalletStore;

#[derive(uniffi::Object)]
pub struct GemAssetDiscoveryService {
    api: Arc<GemDeviceApiClient>,
    assets: Arc<GemAssetsService>,
    wallet_store: Arc<dyn GemWalletStore>,
    store: Arc<dyn GemAssetDiscoveryStore>,
}

#[uniffi::export]
impl GemAssetDiscoveryService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, assets: Arc<GemAssetsService>, wallet_store: Arc<dyn GemWalletStore>, store: Arc<dyn GemAssetDiscoveryStore>) -> Self {
        Self { api, assets, wallet_store, store }
    }

    pub async fn discover(&self, wallet_id: WalletId) -> Result<Vec<AssetId>, GemAssetDiscoveryError> {
        let Some(wallet) = self.wallet_store.get_wallet(wallet_id.clone()).await? else {
            return Ok(vec![]);
        };
        let from_timestamp = self.store.get_assets_timestamp(wallet_id.clone()).await?;
        let timestamp = Utc::now().timestamp() as u64;
        let asset_ids = self.api.client.get_assets_list(wallet_id.id(), from_timestamp).await.map_err(GemApiError::from)?;
        let asset_ids = rules::discoverable_asset_ids(asset_ids, &wallet.accounts);
        if !asset_ids.is_empty() {
            self.assets.prefetch_assets(asset_ids.clone()).await?;
            self.store.enable_assets(wallet_id.clone(), asset_ids.clone()).await?;
        }
        self.store.set_assets_timestamp(wallet_id, timestamp).await?;
        Ok(asset_ids)
    }
}
