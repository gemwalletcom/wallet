use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{Asset, AssetBasic, AssetId, Chain, NFTData, Wallet, WalletType};

use super::GemAssetsService;
use super::model::{GemAssetAction, GemSelectAssetFlow, GemSelectAssetType, GemSelectAssetWalletFlow, GemWalletSearchInput, GemWalletSearchLimits, GemWalletSearchView};
use super::rules;
use crate::services::chain::rules as chain_rules;
use crate::services::nft::model::GemNftEntry;
use crate::services::nft::rules as nft_rules;

use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::perpetual::GemPerpetualService;
use crate::services::preferences::GemPreferencesService;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::search::{GemSearchScope, GemSearchService};
use crate::services::swap::GemSwapService;
use crate::services::transfer::GemRecentActivityService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemAssetSelectionService {
    assets: Arc<GemAssetsService>,
    search: Arc<GemSearchService>,
    balances: Arc<GemBalanceService>,
    price_alerts: Arc<GemPriceAlertService>,
    recent_activity: Arc<GemRecentActivityService>,
    preferences: Arc<GemPreferencesService>,
    perpetuals: Arc<GemPerpetualService>,
    session: Arc<GemWalletSessionService>,
    swap: Arc<GemSwapService>,
}

#[uniffi::export]
impl GemAssetSelectionService {
    #[uniffi::constructor]
    pub fn new(
        assets: Arc<GemAssetsService>,
        search: Arc<GemSearchService>,
        balances: Arc<GemBalanceService>,
        price_alerts: Arc<GemPriceAlertService>,
        recent_activity: Arc<GemRecentActivityService>,
        preferences: Arc<GemPreferencesService>,
        perpetuals: Arc<GemPerpetualService>,
        session: Arc<GemWalletSessionService>,
        swap: Arc<GemSwapService>,
    ) -> Self {
        Self {
            assets,
            search,
            balances,
            price_alerts,
            recent_activity,
            preferences,
            perpetuals,
            session,
            swap,
        }
    }

    pub fn flow(&self, select_type: GemSelectAssetType) -> GemSelectAssetFlow {
        let swap_receive_assets = match &select_type {
            GemSelectAssetType::SwapReceive { pay_asset_id: Some(pay_asset_id) } => Some(self.swap.supported_assets(pay_asset_id.clone())),
            _ => None,
        };
        rules::select_asset_flow(select_type, swap_receive_assets)
    }

    pub fn search_debounce_milliseconds(&self) -> u64 {
        crate::config::search_config::SEARCH_DEBOUNCE_MILLISECONDS
    }

    pub fn wallet_search_limits(&self, query: String) -> GemWalletSearchLimits {
        rules::wallet_search_limits(&query)
    }

    pub fn wallet_search_view(&self, input: GemWalletSearchInput) -> GemWalletSearchView {
        let shows_recents = self.flow(GemSelectAssetType::WalletSearch).shows_recents(!input.query.is_empty(), input.counts.recents > 0);
        let shows_perpetuals = self.show_perpetuals(input.wallet.wallet_type, input.wallet.chains());
        let shows_add_token = self.wallet_flow(GemSelectAssetType::WalletSearch, input.wallet).shows_add_token;
        rules::wallet_search_view(&input.counts, &input.query, input.is_loading, shows_recents, shows_perpetuals, shows_add_token)
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn wallet_flow(&self, select_type: GemSelectAssetType, wallet: Wallet) -> GemSelectAssetWalletFlow {
        let flow = self.flow(select_type);
        let chains = chain_rules::wallet_chains_by_rank(&wallet);
        let has_chains = !chains.is_empty();
        GemSelectAssetWalletFlow {
            shows_add_token: flow.shows_add_token(!rules::token_chains(&wallet).is_empty(), has_chains),
            shows_chain_filter: flow.shows_chain_filter(wallet.wallet_type == WalletType::Multicoin, has_chains),
            chains,
            flow,
        }
    }

    pub fn search_collections(&self, data: Vec<NFTData>, query: String) -> Vec<GemNftEntry> {
        nft_rules::entries(nft_rules::search_collections(data, &query))
    }

    pub fn show_perpetuals(&self, wallet_type: WalletType, chains: Vec<Chain>) -> bool {
        self.preferences.show_perpetuals(wallet_type, chains)
    }

    pub async fn search_assets(&self, query: String) -> Result<Vec<AssetBasic>, GemServiceError> {
        self.search.search_assets(self.session.require_current_wallet().await?, query).await
    }

    pub async fn search(&self, query: String, scope: GemSearchScope) -> Result<bool, GemServiceError> {
        self.search.search(self.session.require_current_wallet().await?, query, scope).await
    }

    pub fn search_key(&self, query: String, scope: GemSearchScope) -> String {
        scope.search_key(query.trim())
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
        let asset_id = asset.id.clone();
        self.recent_activity.add_recent(action, asset).await?;
        self.assets.prepare_for_action(action, asset_id).await
    }

    pub async fn set_price_alert(&self, asset_id: AssetId, enabled: bool) -> Result<(), GemServiceError> {
        self.price_alerts.set_auto_alert(asset_id, enabled).await
    }
}
