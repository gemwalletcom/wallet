use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetBasic, AssetId, PriceAlert, RecentActivityType, Wallet, WalletId};

use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::search::GemSearchService;
use crate::services::transfer::GemRecentActivityService;

#[derive(uniffi::Object)]
pub struct GemAssetSelectionService {
    search: Arc<GemSearchService>,
    balances: Arc<GemBalanceService>,
    price_alerts: Arc<GemPriceAlertService>,
    recent_activity: Arc<GemRecentActivityService>,
    preferences: Arc<GemPreferencesService>,
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
    ) -> Self {
        Self {
            search,
            balances,
            price_alerts,
            recent_activity,
            preferences,
        }
    }

    pub fn currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub async fn search_assets(&self, wallet: Wallet, query: String) -> Result<Vec<AssetBasic>, GemServiceError> {
        self.search.search_assets(wallet, query, self.currency()).await
    }

    pub async fn set_assets_enabled(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.balances.set_assets_enabled(wallet_id, asset_ids, enabled).await
    }

    pub async fn add_recent_asset(&self, activity_type: RecentActivityType, asset_id: AssetId, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.recent_activity.add_asset(activity_type, asset_id, wallet_id).await
    }

    pub async fn set_price_alert(&self, asset_id: AssetId, enabled: bool) -> Result<(), GemServiceError> {
        let alert = PriceAlert::new_auto(asset_id, self.currency());
        match enabled {
            true => self.price_alerts.enable_price_alert(alert).await,
            false => self.price_alerts.delete_price_alerts(vec![alert]).await,
        }
    }
}
