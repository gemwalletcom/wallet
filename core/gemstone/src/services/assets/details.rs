use futures::TryFutureExt;
use std::sync::Arc;

use primitives::{Asset, AssetId, BannerEvent, Deeplink};

use crate::deeplink::GemDeeplinkService;
use crate::models::custom_types::GemBigUint;
use crate::services::balance::GemBalanceService;
use crate::services::banner::{GemBannerContent, GemBannerKey, GemBannerService};
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::stream::GemStreamSubscriptionService;
use crate::services::swap::GemSwapService;
use crate::services::transactions::GemTransactionsService;
use crate::services::wallet_session::GemWalletSessionService;

use crate::services::failures::{StepFailure, record, record_both};

use super::{GemAssetDetails, GemAssetDetailsInput, GemAssetsService, rules};

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

        record_both(
            &mut failures,
            (GemAssetRefreshStep::UpdateBalances, self.balances.update(wallet_id.clone(), vec![asset_id.clone()])),
            (GemAssetRefreshStep::SyncTransactions, self.transactions.sync_wallet(wallet_id, Some(asset_id))),
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

    pub async fn close_banner(&self, key: GemBannerKey) -> Result<(), GemServiceError> {
        self.banners.close(key).await
    }

    pub fn details(&self, input: GemAssetDetailsInput) -> GemAssetDetails {
        let GemAssetDetailsInput {
            wallet_type,
            asset,
            owner_address,
            metadata,
            balance,
            price,
            banner_events,
            price_alerts,
        } = input;
        let chain = asset.chain();
        let has_balance = balance.available > GemBigUint::ZERO;
        GemAssetDetails {
            title: rules::asset_title(&asset),
            state: rules::details_state(wallet_type, chain, &metadata, &balance, &banner_events, price, price_alerts),
            explorer_name: self.explorer.get_explorer_name(chain),
            address_link: owner_address.map(|address| self.explorer.get_address_url(chain, address)),
            token_link: asset.id.token_id.clone().and_then(|token_id| self.explorer.get_token_url(chain, token_id)),
            verification_status: rules::verification_status(&asset, metadata.rank_score),
            network_destination: rules::network_destination(&asset.id),
            share_url: self.deeplinks.build_url(Deeplink::Asset { asset_id: asset.id.clone() }),
            swap_pair: self.swap.pair_for_asset(asset.id, has_balance),
        }
    }

    pub async fn set_price_alert(&self, asset_id: AssetId, enabled: bool) -> Result<(), GemServiceError> {
        self.price_alerts.set_auto_alert(asset_id, enabled).await
    }

    pub async fn sync_price_alerts(&self, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
        self.price_alerts.sync(asset_id).await
    }

    pub fn deeplink_gem_url(&self, deeplink: Deeplink) -> String {
        self.deeplinks.build_gem_url(deeplink)
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::Chain;

    use super::super::details_testkit::AssetDetailsTestkit;
    use super::*;

    #[test]
    fn test_every_step_is_attempted_and_reported_on_its_own() {
        block_on(async {
            let testkit = AssetDetailsTestkit::with_status(503);

            let failures = testkit.service.refresh(Chain::Ethereum.as_asset_id()).await;

            let steps: Vec<GemAssetRefreshStep> = failures.iter().map(|failure| failure.step).collect();
            assert!(steps.contains(&GemAssetRefreshStep::SyncAsset), "{failures:?}");
            assert!(steps.contains(&GemAssetRefreshStep::UpdateBalances), "{failures:?}");
            assert!(steps.contains(&GemAssetRefreshStep::SyncTransactions), "{failures:?}");
        })
    }

    #[test]
    fn test_no_current_wallet_reports_one_failure_and_asks_for_nothing() {
        block_on(async {
            let testkit = AssetDetailsTestkit::with_status(503);
            testkit.discovery.session.set_current_wallet_id(None).unwrap();

            let failures = testkit.service.refresh(Chain::Ethereum.as_asset_id()).await;

            assert_eq!(failures.len(), 1);
            assert_eq!(failures[0].step, GemAssetRefreshStep::UpdateBalances);
            assert!(testkit.provider.requested_paths().is_empty());
        })
    }

    #[test]
    fn test_an_asset_that_loads_leaves_the_other_steps_to_fail_alone() {
        block_on(async {
            let asset = serde_json::to_string(&primitives::AssetFull::mock()).unwrap();
            let testkit = AssetDetailsTestkit::with_bodies(200, &[("assets/", &asset)]);

            let failures = testkit.service.refresh(Chain::Ethereum.as_asset_id()).await;

            let steps: Vec<GemAssetRefreshStep> = failures.iter().map(|failure| failure.step).collect();
            assert!(!steps.contains(&GemAssetRefreshStep::SyncAsset), "{failures:?}");
            assert!(steps.contains(&GemAssetRefreshStep::UpdateBalances), "{failures:?}");
            assert!(steps.contains(&GemAssetRefreshStep::SyncTransactions), "{failures:?}");
        })
    }
}
