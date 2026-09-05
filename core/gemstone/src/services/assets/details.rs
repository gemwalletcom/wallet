use futures::TryFutureExt;
use std::sync::Arc;

use primitives::{Asset, AssetId, BannerEvent, Chain, Deeplink, VerificationStatus};

use crate::deeplink::GemDeeplinkService;
use crate::services::balance::GemBalanceService;
use crate::services::banner::{GemBannerAction, GemBannerContent, GemBannerKey, GemBannerService};
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::stream::GemStreamSubscriptionService;
use crate::services::swap::{GemSwapPairSuggestion, GemSwapService};
use crate::services::transactions::GemTransactionsService;
use crate::services::wallet_session::GemWalletSessionService;
use primitives::BlockExplorerLink;

use crate::services::failures::{StepFailure, record};

use super::{GemAssetNetworkDestination, GemAssetsService, rules};

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetRefreshStep {
    AddPrices,
    SyncAsset,
    SyncAssociations,
    UpdateBalances,
    SyncTransactions,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemAssetRefreshFailure {
    pub step: GemAssetRefreshStep,
    pub message: String,
}

impl StepFailure for GemAssetRefreshFailure {
    type Step = GemAssetRefreshStep;

    fn new(step: GemAssetRefreshStep, message: String) -> Self {
        Self { step, message }
    }
}

#[derive(uniffi::Object)]
pub struct GemAssetDetailsService {
    assets: Arc<GemAssetsService>,
    balances: Arc<GemBalanceService>,
    transactions: Arc<GemTransactionsService>,
    banners: Arc<GemBannerService>,
    swap: Arc<GemSwapService>,
    explorer: Arc<GemExplorerService>,
    price_alerts: Arc<GemPriceAlertService>,
    stream: Arc<GemStreamSubscriptionService>,
    deeplinks: Arc<GemDeeplinkService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemAssetDetailsService {
    #[uniffi::constructor]
    pub fn new(
        assets: Arc<GemAssetsService>,
        balances: Arc<GemBalanceService>,
        transactions: Arc<GemTransactionsService>,
        banners: Arc<GemBannerService>,
        swap: Arc<GemSwapService>,
        explorer: Arc<GemExplorerService>,
        price_alerts: Arc<GemPriceAlertService>,
        stream: Arc<GemStreamSubscriptionService>,
        deeplinks: Arc<GemDeeplinkService>,
        session: Arc<GemWalletSessionService>,
    ) -> Self {
        Self {
            assets,
            balances,
            transactions,
            banners,
            swap,
            explorer,
            price_alerts,
            stream,
            deeplinks,
            session,
        }
    }

    pub async fn refresh(&self, asset_id: AssetId) -> Vec<GemAssetRefreshFailure> {
        let mut failures = Vec::new();
        let wallet_id = match self.session.current_wallet_id() {
            Ok(wallet_id) => wallet_id,
            Err(error) => return vec![GemAssetRefreshFailure::new(GemAssetRefreshStep::UpdateBalances, error.to_string())],
        };
        record(&mut failures, GemAssetRefreshStep::AddPrices, self.stream.add_prices(vec![asset_id.clone()])).await;

        let associations = match self.assets.sync_asset(asset_id.clone()).await {
            Ok(asset) => asset.associations.into_iter().map(|association| association.asset_id).collect(),
            Err(error) => {
                failures.push(GemAssetRefreshFailure::new(GemAssetRefreshStep::SyncAsset, error.to_string()));
                Vec::new()
            }
        };
        if !associations.is_empty() {
            record(
                &mut failures,
                GemAssetRefreshStep::SyncAssociations,
                self.assets.sync_missing_assets(associations).map_ok(|_| ()),
            )
            .await;
        }

        record(
            &mut failures,
            GemAssetRefreshStep::UpdateBalances,
            self.balances.update(wallet_id.clone(), vec![asset_id.clone()]),
        )
        .await;
        record(
            &mut failures,
            GemAssetRefreshStep::SyncTransactions,
            self.transactions.sync_wallet(wallet_id, Some(asset_id)),
        )
        .await;
        failures
    }

    pub async fn sync_transactions(&self, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
        self.transactions.sync_wallet(self.session.current_wallet_id()?, asset_id).await
    }

    pub async fn update_balances(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.balances.update(self.session.current_wallet_id()?, asset_ids).await
    }

    pub async fn set_asset_pinned(&self, asset_id: AssetId, pinned: bool) -> Result<(), GemServiceError> {
        self.balances.set_asset_pinned(self.session.current_wallet_id()?, asset_id, pinned).await
    }

    pub async fn set_assets_enabled(&self, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.balances.set_assets_enabled(self.session.current_wallet_id()?, asset_ids, enabled).await
    }

    pub async fn add_prices(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.stream.add_prices(asset_ids).await
    }

    pub fn banner_content(&self, event: BannerEvent, asset: Option<Asset>) -> GemBannerContent {
        self.banners.banner_content(event, asset)
    }

    pub async fn apply_banner_action(&self, key: GemBannerKey, action: GemBannerAction) -> Result<(), GemServiceError> {
        self.banners.apply_action(key, action).await
    }

    pub fn network_destination(&self, asset_id: AssetId) -> Option<GemAssetNetworkDestination> {
        rules::network_destination(&asset_id)
    }

    pub fn verification_status(&self, asset: Asset, rank: i32) -> Option<VerificationStatus> {
        rules::verification_status(&asset, rank)
    }

    pub fn swap_pair(&self, asset_id: AssetId, has_balance: bool) -> GemSwapPairSuggestion {
        self.swap.pair_for_asset(asset_id, has_balance)
    }

    pub fn explorer_name(&self, chain: Chain) -> String {
        self.explorer.get_explorer_name(chain)
    }

    pub fn address_url(&self, chain: Chain, address: String) -> BlockExplorerLink {
        self.explorer.get_address_url(chain, address)
    }

    pub fn token_url(&self, chain: Chain, address: String) -> Option<BlockExplorerLink> {
        self.explorer.get_token_url(chain, address)
    }

    pub async fn set_price_alert(&self, asset_id: AssetId, enabled: bool) -> Result<(), GemServiceError> {
        self.price_alerts.set_auto_alert(asset_id, enabled).await
    }

    pub async fn sync_price_alerts(&self, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
        self.price_alerts.sync(asset_id).await
    }

    pub fn deeplink_url(&self, deeplink: Deeplink) -> String {
        self.deeplinks.build_url(deeplink)
    }

    pub fn deeplink_gem_url(&self, deeplink: Deeplink) -> String {
        self.deeplinks.build_gem_url(deeplink)
    }
}
