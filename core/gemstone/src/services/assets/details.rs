use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{Asset, AssetFull, AssetId, BannerEvent, Chain, Deeplink, PriceAlert, WalletId};

use crate::block_explorer::GemBlockExplorerLink;
use crate::deeplink::GemDeeplinkService;
use crate::services::balance::GemBalanceService;
use crate::services::banner::{GemBannerAction, GemBannerContent, GemBannerKey, GemBannerService};
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::stream::GemStreamSubscriptionService;
use crate::services::swap::{GemSwapPairSuggestion, GemSwapService};
use crate::services::transactions::GemTransactionsService;

use super::GemAssetsService;

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
        }
    }

    pub async fn sync_asset(&self, asset_id: AssetId, currency: Currency) -> Result<AssetFull, GemServiceError> {
        self.assets.sync_asset(asset_id, currency).await
    }

    pub async fn sync_missing_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        self.assets.sync_missing_assets(asset_ids).await
    }

    pub async fn sync_transactions(&self, wallet_id: WalletId, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
        self.transactions.sync(wallet_id, asset_id).await
    }

    pub async fn update_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.balances.update(wallet_id, asset_ids).await
    }

    pub async fn set_asset_pinned(&self, wallet_id: WalletId, asset_id: AssetId, pinned: bool) -> Result<(), GemServiceError> {
        self.balances.set_asset_pinned(wallet_id, asset_id, pinned).await
    }

    pub async fn set_assets_enabled(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.balances.set_assets_enabled(wallet_id, asset_ids, enabled).await
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

    pub fn swap_pair(&self, asset_id: AssetId, has_balance: bool) -> GemSwapPairSuggestion {
        self.swap.pair_for_asset(asset_id, has_balance)
    }

    pub fn address_url(&self, chain: Chain, address: String) -> GemBlockExplorerLink {
        self.explorer.get_address_url(chain, address)
    }

    pub fn token_url(&self, chain: Chain, address: String) -> Option<GemBlockExplorerLink> {
        self.explorer.get_token_url(chain, address)
    }

    pub async fn enable_price_alert(&self, alert: PriceAlert) -> Result<(), GemServiceError> {
        self.price_alerts.enable_price_alert(alert).await
    }

    pub async fn delete_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), GemServiceError> {
        self.price_alerts.delete_price_alerts(alerts).await
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
