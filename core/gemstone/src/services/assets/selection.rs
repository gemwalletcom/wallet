use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{Asset, AssetBasic, AssetId, PerpetualId, Wallet, WalletType};

use super::model::{GemAssetAction, GemSelectAssetFlow, GemSelectAssetType, GemSelectAssetWalletFlow, GemWalletSearchInput, GemWalletSearchLimits, GemWalletSearchResultsInput, GemWalletSearchResultsView, GemWalletSearchView};
use super::rules;
use crate::services::chain::rules as chain_rules;

use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::perpetual::GemPerpetualService;
use crate::services::preferences::GemPreferencesService;
use crate::services::search::{GemSearchScope, GemSearchService};
use crate::services::swap::GemSwapService;
use crate::services::toast::GemToast;
use crate::services::transfer::GemRecentActivityService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemAssetSelectionService {
    search: Arc<GemSearchService>,
    balances: Arc<GemBalanceService>,
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
        search: Arc<GemSearchService>,
        balances: Arc<GemBalanceService>,
        recent_activity: Arc<GemRecentActivityService>,
        preferences: Arc<GemPreferencesService>,
        perpetuals: Arc<GemPerpetualService>,
        session: Arc<GemWalletSessionService>,
        swap: Arc<GemSwapService>,
    ) -> Self {
        Self {
            search,
            balances,
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

    pub fn wallet_search_limits(&self, query: String) -> GemWalletSearchLimits {
        rules::wallet_search_limits(&query)
    }

    pub fn wallet_search_view(&self, input: GemWalletSearchInput) -> GemWalletSearchView {
        let shows_recents = self.flow(GemSelectAssetType::WalletSearch).shows_recents(!input.query.is_empty(), input.recents > 0);
        let shows_perpetuals = self.shows_perpetuals(&input.wallet);
        let shows_add_token = self.wallet_flow(GemSelectAssetType::WalletSearch, input.wallet.clone()).shows_add_token;
        rules::wallet_search_view(input, shows_recents, shows_perpetuals, shows_add_token)
    }

    pub fn wallet_search_results_view(&self, input: GemWalletSearchResultsInput) -> GemWalletSearchResultsView {
        let shows_perpetuals = self.shows_perpetuals(&input.wallet);
        rules::wallet_search_results_view(input, shows_perpetuals)
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn wallet_flow(&self, select_type: GemSelectAssetType, wallet: Wallet) -> GemSelectAssetWalletFlow {
        let flow = self.flow(select_type);
        let chains = chain_rules::wallet_chains_by_rank(&wallet);
        let has_chains = !chains.is_empty();
        let shows_add_token = flow.shows_add_token(!rules::token_chains(&wallet).is_empty(), has_chains);
        GemSelectAssetWalletFlow {
            empty_state: rules::search_assets_empty_state(shows_add_token),
            shows_add_token,
            shows_chain_filter: flow.shows_chain_filter(wallet.wallet_type == WalletType::Multicoin, has_chains),
            chains,
            flow,
        }
    }

    pub async fn search_assets(&self, query: String) -> Result<Vec<AssetBasic>, GemServiceError> {
        self.search.search_assets(self.session.require_current_wallet().await?, query).await
    }

    pub async fn search(&self, query: String, scope: GemSearchScope) -> Result<bool, GemServiceError> {
        self.search.search(self.session.require_current_wallet().await?, query, scope).await
    }

    pub async fn set_assets_enabled(&self, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.balances.set_assets_enabled(self.session.current_wallet_id()?, asset_ids, enabled).await
    }

    pub async fn set_asset_pinned(&self, asset: Asset, pinned: bool) -> Result<GemToast, GemServiceError> {
        self.balances.set_asset_pinned(self.session.current_wallet_id()?, asset.id, pinned).await?;
        Ok(GemToast::pinned(asset.name, pinned))
    }

    pub async fn set_perpetual_pinned(&self, perpetual_id: PerpetualId, name: String, pinned: bool) -> Result<GemToast, GemServiceError> {
        self.perpetuals.set_pinned(perpetual_id, pinned).await?;
        Ok(GemToast::pinned(name, pinned))
    }

    pub async fn add_recent(&self, action: GemAssetAction, asset: Asset) -> Result<(), GemServiceError> {
        self.recent_activity.add_recent(action, asset).await
    }
}

impl GemAssetSelectionService {
    fn shows_perpetuals(&self, wallet: &Wallet) -> bool {
        self.preferences.show_perpetuals(wallet.wallet_type, wallet.chains())
    }
}
