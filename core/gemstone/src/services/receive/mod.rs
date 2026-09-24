pub mod model;
pub mod rules;

use std::sync::Arc;

use primitives::{Asset, AssetId, Chain, Wallet, WalletId};

use crate::services::assets::{GemAssetAction, GemAssetsService};
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::transfer::GemRecentActivityService;
pub use model::{GemReceiveNetworks, GemReceiveWarning};

#[derive(uniffi::Object)]
pub struct GemReceiveService {
    balances: Arc<GemBalanceService>,
    assets: Arc<GemAssetsService>,
    recent_activity: Arc<GemRecentActivityService>,
}

#[uniffi::export]
impl GemReceiveService {
    #[uniffi::constructor]
    pub fn new(balances: Arc<GemBalanceService>, assets: Arc<GemAssetsService>, recent_activity: Arc<GemRecentActivityService>) -> Self {
        Self { balances, assets, recent_activity }
    }

    pub fn warnings(&self, chain: Chain) -> Vec<GemReceiveWarning> {
        rules::warnings(chain)
    }

    pub fn networks(&self, asset_id: AssetId, associations: Vec<AssetId>, wallet: Wallet) -> GemReceiveNetworks {
        rules::networks(asset_id, associations, &wallet)
    }

    pub async fn enable_asset(&self, wallet_id: WalletId, asset_id: AssetId) -> Result<(), GemServiceError> {
        self.balances.enable_assets(wallet_id, vec![asset_id.clone()]).await?;
        if let Ok(asset) = self.asset(asset_id).await {
            let _ = self.recent_activity.add_recent(GemAssetAction::Receive, asset).await;
        }
        Ok(())
    }

    pub async fn asset(&self, asset_id: AssetId) -> Result<Asset, GemServiceError> {
        self.assets.ensure_asset(asset_id).await
    }
}
