use futures::TryFutureExt;
use std::sync::Arc;

use primitives::{AssetId, BannerEvent, Deeplink};

use crate::deeplink::GemDeeplinkService;
use crate::models::custom_types::GemBigUint;
use crate::models::state::GemLoadState;
use crate::services::balance::GemBalanceService;
use crate::services::banner::{GemBannerContext, GemBannerKey, GemBannerService};
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::stream::GemStreamSubscriptionService;
use crate::services::swap::GemSwapService;
use crate::services::transactions::GemTransactionsService;
use crate::services::wallet_session::GemWalletSessionService;

use crate::services::failures::{StepFailure, record, record_result};

use super::{GemAssetDetails, GemAssetDetailsInput, GemAssetsService, rules};

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetRefreshStep {
    AddPrices,
    SyncAsset,
    SyncPriceAlerts,
    UpdateBalances,
    SyncTransactions,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemAssetRefreshFailure {
    pub step: GemAssetRefreshStep,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetRefresh {
    pub transactions: GemLoadState,
    pub failures: Vec<GemAssetRefreshFailure>,
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

    pub async fn refresh(&self, asset_id: AssetId, has_transactions: bool) -> GemAssetRefresh {
        let mut failures = Vec::new();
        let wallet_id = match self.session.current_wallet_id() {
            Ok(wallet_id) => wallet_id,
            Err(error) => {
                return GemAssetRefresh {
                    transactions: GemLoadState::refreshed(Err(error.clone()), has_transactions),
                    failures: vec![GemAssetRefreshFailure::new(GemAssetRefreshStep::UpdateBalances, error.to_string())],
                };
            }
        };
        record(&mut failures, GemAssetRefreshStep::AddPrices, self.stream.add_prices(vec![asset_id.clone()])).await;

        record(&mut failures, GemAssetRefreshStep::SyncAsset, self.assets.sync_asset_associations(asset_id.clone()).map_ok(|_| ())).await;

        let (balances, transactions, price_alerts) = futures::join!(
            self.balances.update(wallet_id.clone(), vec![asset_id.clone()]),
            self.transactions.sync_wallet(wallet_id, Some(asset_id.clone())),
            self.price_alerts.sync(Some(asset_id))
        );
        record_result(&mut failures, GemAssetRefreshStep::SyncPriceAlerts, price_alerts);
        record_result(&mut failures, GemAssetRefreshStep::UpdateBalances, balances);
        record_result(&mut failures, GemAssetRefreshStep::SyncTransactions, transactions.clone());
        GemAssetRefresh {
            transactions: GemLoadState::refreshed(transactions, has_transactions),
            failures,
        }
    }

    pub async fn set_asset_pinned(&self, asset_id: AssetId, pinned: bool) -> Result<(), GemServiceError> {
        self.balances.set_asset_pinned(self.session.current_wallet_id()?, asset_id, pinned).await
    }

    pub async fn set_assets_enabled(&self, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.balances.set_assets_enabled(self.session.current_wallet_id()?, asset_ids, enabled).await
    }

    pub async fn close_banner(&self, key: GemBannerKey) -> Result<(), GemServiceError> {
        self.banners.close(key).await
    }

    pub fn details(&self, input: GemAssetDetailsInput) -> GemAssetDetails {
        let GemAssetDetailsInput {
            wallet,
            asset,
            owner_address,
            metadata,
            balance,
            price,
            price_change_percentage_24h,
            currency,
            banners,
            price_alerts,
            fee_balance_metadata,
        } = input;
        let wallet_type = wallet.wallet_type;
        let visible_banners = self.banners.visible_banners(&GemBannerContext::asset(Some(wallet), asset.clone(), &metadata, &balance), banners);
        let banner_events: Vec<BannerEvent> = visible_banners.iter().map(|row| row.banner.event).collect();
        let chain = asset.chain();
        let has_balance = balance.available > GemBigUint::ZERO;
        GemAssetDetails {
            title: rules::asset_title(&asset),
            balance_value: crate::services::balance::rules::balance_amount(&balance.total(), &asset),
            fiat_value: rules::fiat_value(&asset, &balance, price, currency.clone()),
            state: rules::details_state(wallet_type, &metadata, &banner_events, &price_alerts),
            visible_banners,
            sections: rules::details_sections(rules::DetailsSectionsInput {
                wallet_type,
                asset: &asset,
                metadata: &metadata,
                balance: &balance,
                price,
                price_change_percentage_24h,
                currency,
                price_alerts: &price_alerts,
                fee_balance_metadata,
            }),
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
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::Chain;

    use super::super::testkit::AssetDetailsTestkit;
    use super::*;

    #[test]
    fn test_every_step_is_attempted_and_reported_on_its_own() {
        block_on(async {
            let testkit = AssetDetailsTestkit::with_status(503);

            let refresh = testkit.service.refresh(Chain::Ethereum.as_asset_id(), false).await;
            let failures = refresh.failures;

            let steps: Vec<GemAssetRefreshStep> = failures.iter().map(|failure| failure.step).collect();
            assert!(steps.contains(&GemAssetRefreshStep::SyncAsset), "{failures:?}");
            assert!(steps.contains(&GemAssetRefreshStep::SyncPriceAlerts), "{failures:?}");
            assert!(steps.contains(&GemAssetRefreshStep::UpdateBalances), "{failures:?}");
            assert!(steps.contains(&GemAssetRefreshStep::SyncTransactions), "{failures:?}");
            assert!(matches!(refresh.transactions, GemLoadState::Error { .. }), "a failed sync with nothing stored shows the error");
            assert_eq!(
                testkit.service.refresh(Chain::Ethereum.as_asset_id(), true).await.transactions,
                GemLoadState::Data,
                "a failed sync keeps the transactions already shown"
            );
        })
    }

    #[test]
    fn test_no_current_wallet_reports_one_failure_and_asks_for_nothing() {
        block_on(async {
            let testkit = AssetDetailsTestkit::with_status(503);
            testkit.discovery.session.set_current_wallet_id(None).unwrap();

            let failures = testkit.service.refresh(Chain::Ethereum.as_asset_id(), true).await.failures;

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

            let failures = testkit.service.refresh(Chain::Ethereum.as_asset_id(), true).await.failures;

            let steps: Vec<GemAssetRefreshStep> = failures.iter().map(|failure| failure.step).collect();
            assert!(!steps.contains(&GemAssetRefreshStep::SyncAsset), "{failures:?}");
            assert!(steps.contains(&GemAssetRefreshStep::UpdateBalances), "{failures:?}");
            assert!(steps.contains(&GemAssetRefreshStep::SyncTransactions), "{failures:?}");
        })
    }
}
