use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{Asset, AssetBasic, AssetId, Chain, NFTData};

use super::model::GemAssetAction;
use super::rules;
use crate::services::chain::rules as chain_rules;
use crate::services::nft::GemNftSearchItem;
use crate::services::nft::rules as nft_rules;

use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::perpetual::GemPerpetualService;
use crate::services::preferences::GemPreferencesService;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::search::{GemSearchScope, GemSearchService};
use crate::services::transfer::GemRecentActivityService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemAssetSelectionService {
    search: Arc<GemSearchService>,
    balances: Arc<GemBalanceService>,
    price_alerts: Arc<GemPriceAlertService>,
    recent_activity: Arc<GemRecentActivityService>,
    preferences: Arc<GemPreferencesService>,
    perpetuals: Arc<GemPerpetualService>,
    session: Arc<GemWalletSessionService>,
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
        session: Arc<GemWalletSessionService>,
    ) -> Self {
        Self {
            search,
            balances,
            price_alerts,
            recent_activity,
            preferences,
            perpetuals,
            session,
        }
    }

    pub fn currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn filter_chains(&self) -> Result<Vec<Chain>, GemServiceError> {
        Ok(chain_rules::wallet_chains_by_rank(&self.session.current_wallet()?))
    }

    pub fn supports_tokens(&self) -> bool {
        self.session
            .get_current_wallet()
            .ok()
            .flatten()
            .is_some_and(|wallet| !rules::token_chains(&wallet).is_empty())
    }

    pub fn search_collections(&self, data: Vec<NFTData>, query: String) -> Vec<GemNftSearchItem> {
        nft_rules::search_collections(data, &query)
    }

    pub fn show_perpetuals(&self) -> bool {
        self.session
            .get_current_wallet()
            .ok()
            .flatten()
            .is_some_and(|wallet| self.preferences.show_perpetuals(wallet))
    }

    pub async fn search_assets(&self, query: String) -> Result<Vec<AssetBasic>, GemServiceError> {
        self.search.search_assets(self.session.current_wallet()?, query, self.currency()).await
    }

    pub async fn search(&self, query: String, scope: GemSearchScope) -> Result<bool, GemServiceError> {
        self.search.search(self.session.current_wallet()?, query, scope, self.currency()).await
    }

    pub async fn set_assets_enabled(&self, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.balances.set_assets_enabled(self.session.current_wallet_id()?, asset_ids, enabled).await
    }

    pub async fn set_asset_pinned(&self, asset_id: AssetId, pinned: bool) -> Result<(), GemServiceError> {
        self.balances.set_asset_pinned(self.session.current_wallet_id()?, asset_id, pinned).await
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
