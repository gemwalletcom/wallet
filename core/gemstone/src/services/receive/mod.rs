pub mod model;
pub mod rules;

use std::sync::Arc;

use primitives::{Asset, AssetId, Chain, Wallet, WalletId};

use crate::services::assets::GemAssetsService;
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
pub use model::GemReceiveWarning;

#[derive(uniffi::Object)]
pub struct GemReceiveService {
    balances: Arc<GemBalanceService>,
    assets: Arc<GemAssetsService>,
}

#[uniffi::export]
impl GemReceiveService {
    #[uniffi::constructor]
    pub fn new(balances: Arc<GemBalanceService>, assets: Arc<GemAssetsService>) -> Self {
        Self { balances, assets }
    }

    pub fn warnings(&self, chain: Chain) -> Vec<GemReceiveWarning> {
        rules::warnings(chain)
    }

    pub fn network_asset_ids(&self, asset_id: AssetId, associations: Vec<AssetId>, wallet: Wallet) -> Vec<AssetId> {
        rules::network_asset_ids(asset_id, associations, &wallet)
    }

    pub async fn enable_asset(&self, wallet_id: WalletId, asset_id: AssetId) -> Result<(), GemServiceError> {
        self.balances.set_assets_enabled(wallet_id, vec![asset_id], true).await
    }

    pub async fn sync_network_asset_ids(&self, asset_id: AssetId, wallet: Wallet) -> Result<Vec<AssetId>, GemServiceError> {
        let associations = self.assets.sync_asset_associations(asset_id.clone()).await?;
        Ok(rules::network_asset_ids(asset_id, associations, &wallet))
    }

    pub async fn asset(&self, asset_id: AssetId) -> Result<Asset, GemServiceError> {
        self.assets.ensure_asset(asset_id).await
    }
}
