use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{Asset, AssetBasic, AssetId, Wallet, WalletId};

use super::model::GemAssetAction;

use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::perpetual::GemPerpetualService;
use crate::services::preferences::GemPreferencesService;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::search::{GemSearchScope, GemSearchService};
use crate::services::transfer::GemRecentActivityService;

#[derive(uniffi::Object)]
pub struct GemAssetSelectionService {
    search: Arc<GemSearchService>,
    balances: Arc<GemBalanceService>,
    price_alerts: Arc<GemPriceAlertService>,
    recent_activity: Arc<GemRecentActivityService>,
    preferences: Arc<GemPreferencesService>,
    perpetuals: Arc<GemPerpetualService>,
}

#[uniffi::export]
impl GemAssetSelectionService {
    #[uniffi::constructor]
    pub fn new(
        search: Arc<GemSearchService>,
        balances: Arc<GemBalanceService>,
        price_alerts: Arc<GemPriceAlertService>,
        recent_activity: Arc<GemRecentActivityService>,
        preferences: Arc<GemPreferencesService>,
        perpetuals: Arc<GemPerpetualService>,
    ) -> Self {
        Self {
            search,
            balances,
            price_alerts,
            recent_activity,
            preferences,
            perpetuals,
        }
    }

    pub fn currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn show_perpetuals(&self, wallet: Wallet) -> bool {
        self.preferences.show_perpetuals(wallet)
    }

    pub async fn search_assets(&self, wallet: Wallet, query: String) -> Result<Vec<AssetBasic>, GemServiceError> {
        self.search.search_assets(wallet, query, self.currency()).await
    }

    pub async fn search(&self, wallet: Wallet, query: String, scope: GemSearchScope) -> Result<bool, GemServiceError> {
        self.search.search(wallet, query, scope, self.currency()).await
    }

    pub async fn set_assets_enabled(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.balances.set_assets_enabled(wallet_id, asset_ids, enabled).await
    }

    pub async fn set_asset_pinned(&self, wallet_id: WalletId, asset_id: AssetId, pinned: bool) -> Result<(), GemServiceError> {
        self.balances.set_asset_pinned(wallet_id, asset_id, pinned).await
    }

    pub async fn set_perpetual_pinned(&self, perpetual_id: String, pinned: bool) -> Result<(), GemServiceError> {
        self.perpetuals.set_pinned(perpetual_id, pinned).await
    }

    pub async fn add_recent(&self, action: GemAssetAction, asset: Asset) -> Result<(), GemServiceError> {
        self.recent_activity.add_recent(action, asset).await
    }

    pub async fn set_price_alert(&self, asset_id: AssetId, enabled: bool) -> Result<(), GemServiceError> {
        self.price_alerts.set_auto_alert(asset_id, enabled).await
    }
}
