mod rules;

use std::sync::Arc;

use primitives::{Asset, AssetId, BannerEvent, WalletId};

use crate::services::asset_discovery::GemAssetDiscoveryService;
use crate::services::balance::GemBalanceService;
use crate::services::banner::{GemBannerAction, GemBannerContent, GemBannerKey, GemBannerService};
use crate::services::error::GemServiceError;
use crate::services::wallet_preferences::{GemDiscoveryStep, GemWalletPreferencesService};

#[derive(uniffi::Object)]
pub struct GemWalletHomeService {
    balances: Arc<GemBalanceService>,
    discovery: Arc<GemAssetDiscoveryService>,
    banners: Arc<GemBannerService>,
    wallet_preferences: Arc<GemWalletPreferencesService>,
}

#[uniffi::export]
impl GemWalletHomeService {
    #[uniffi::constructor]
    pub fn new(
        balances: Arc<GemBalanceService>,
        discovery: Arc<GemAssetDiscoveryService>,
        banners: Arc<GemBannerService>,
        wallet_preferences: Arc<GemWalletPreferencesService>,
    ) -> Self {
        Self {
            balances,
            discovery,
            banners,
            wallet_preferences,
        }
    }

    pub fn includes_perpetual_collateral(&self, wallet_id: WalletId) -> bool {
        self.wallet_preferences.includes_perpetual_collateral(wallet_id)
    }

    pub fn shows_initial_loading(&self, wallet_id: WalletId) -> Result<bool, GemServiceError> {
        let completed = self.wallet_preferences.is_initial_load_completed(wallet_id.clone(), GemDiscoveryStep::Assets)?;
        Ok(rules::shows_initial_loading(completed, self.wallet_preferences.get_assets_timestamp(wallet_id)))
    }

    pub async fn refresh(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        let (balances, discovery) = futures::join!(self.balances.update(wallet_id.clone(), asset_ids), self.discovery.discover(wallet_id));
        balances?;
        discovery.map(|_| ())
    }

    pub async fn set_asset_pinned(&self, wallet_id: WalletId, asset_id: AssetId, pinned: bool) -> Result<(), GemServiceError> {
        self.balances.set_asset_pinned(wallet_id, asset_id, pinned).await
    }

    pub async fn set_assets_enabled(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.balances.set_assets_enabled(wallet_id, asset_ids, enabled).await
    }

    pub fn banner_content(&self, event: BannerEvent, asset: Option<Asset>) -> GemBannerContent {
        self.banners.banner_content(event, asset)
    }

    pub async fn apply_banner_action(&self, key: GemBannerKey, action: GemBannerAction) -> Result<(), GemServiceError> {
        self.banners.apply_action(key, action).await
    }
}
