use std::sync::Arc;

use primitives::chart::ChartCandleUpdate;
use primitives::{Asset, AssetId, Chain, ChartPeriod, Perpetual, PerpetualPosition, TransactionType};

use super::candles::{GemCandleRequest, GemCandleResult};
use super::model::{GemPerpetualDetails, GemPerpetualPositionAction, GemPerpetualPositionKind, GemPerpetualRefreshFailure, GemPerpetualRefreshStep};
use super::{GemPerpetualService, rules};
use crate::models::perpetual::{GemChartCandleStick, GemPerpetualSubscription};
use crate::models::state::GemLoadState;
use crate::services::error::GemServiceError;
use crate::services::failures::record_result;
use crate::services::preferences::GemPreferencesService;
use crate::services::transactions::{GemTransactionFilter, GemTransactionsService, rules as transaction_rules};
use crate::services::transfer::GemTransferData;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemPerpetualDetailsService {
    perpetuals: Arc<GemPerpetualService>,
    transactions: Arc<GemTransactionsService>,
    preferences: Arc<GemPreferencesService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemPerpetualDetailsService {
    #[uniffi::constructor]
    pub fn new(perpetuals: Arc<GemPerpetualService>, transactions: Arc<GemTransactionsService>, preferences: Arc<GemPreferencesService>, session: Arc<GemWalletSessionService>) -> Self {
        Self {
            perpetuals,
            transactions,
            preferences,
            session,
        }
    }

    pub fn activity_types(&self) -> Vec<TransactionType> {
        transaction_rules::filter_transaction_types(GemTransactionFilter::Perpetuals)
    }

    pub fn details(&self, perpetual: Perpetual, asset: Asset, positions: Vec<PerpetualPosition>) -> GemPerpetualDetails {
        rules::details(&perpetual, &asset, positions)
    }

    pub fn position_action(&self, perpetual: Perpetual, asset: Asset, position: Option<PerpetualPosition>, kind: GemPerpetualPositionKind) -> Result<GemPerpetualPositionAction, GemServiceError> {
        rules::position_action(&perpetual, &asset, position, kind)
    }

    pub fn close_transfer(&self, perpetual: Perpetual, asset: Asset, position: Option<PerpetualPosition>) -> Result<GemTransferData, GemServiceError> {
        rules::close_transfer(&perpetual, &asset, position)
    }

    pub fn chart_period(&self) -> ChartPeriod {
        self.preferences.get_perpetual_chart_period()
    }

    pub fn set_chart_period(&self, period: ChartPeriod) -> Result<(), GemServiceError> {
        self.preferences.set_perpetual_chart_period(period)
    }

    pub fn candle_subscription(&self, perpetual: Perpetual, period: ChartPeriod) -> GemPerpetualSubscription {
        GemPerpetualSubscription::Candle {
            symbol: rules::symbol(&perpetual),
            interval: rules::candle_interval(&period).to_string(),
        }
    }

    pub fn market_subscription(&self, perpetual: Perpetual) -> GemPerpetualSubscription {
        GemPerpetualSubscription::MarketData { symbol: rules::symbol(&perpetual) }
    }

    pub async fn candles(&self, request: GemCandleRequest) -> GemCandleResult {
        let candles = self.perpetuals.get_candlesticks(Chain::HyperCore, request.symbol.clone(), request.period).await;
        GemCandleResult {
            request,
            state: GemLoadState::of(&candles),
            candles: candles.unwrap_or_default(),
        }
    }

    pub async fn candlesticks(&self, perpetual: Perpetual, period: ChartPeriod) -> Result<Vec<GemChartCandleStick>, GemServiceError> {
        self.perpetuals.get_candlesticks(Chain::HyperCore, rules::symbol(&perpetual), period).await
    }

    pub fn merged_candles(&self, candles: Vec<GemChartCandleStick>, update: ChartCandleUpdate, perpetual: Perpetual, period: ChartPeriod) -> Option<Vec<GemChartCandleStick>> {
        rules::merged_candles(candles, update, &perpetual, &period)
    }

    pub async fn refresh(&self, asset_id: AssetId) -> Vec<GemPerpetualRefreshFailure> {
        let mut failures = Vec::new();
        let (positions, transactions) = futures::join!(self.sync_positions(), self.sync_transactions(asset_id));
        record_result(&mut failures, GemPerpetualRefreshStep::Positions, positions);
        record_result(&mut failures, GemPerpetualRefreshStep::Transactions, transactions);
        failures
    }
}

impl GemPerpetualDetailsService {
    async fn sync_positions(&self) -> Result<(), GemServiceError> {
        self.perpetuals.sync_current_positions().await
    }

    async fn sync_transactions(&self, asset_id: AssetId) -> Result<(), GemServiceError> {
        self.transactions.sync_wallet(self.session.current_wallet_id()?, Some(asset_id)).await
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::Chain;

    use super::super::testkit::PerpetualTestkit;
    use super::*;

    #[test]
    fn test_every_refresh_asks_for_the_transactions_of_the_asset_it_names() {
        block_on(async {
            let testkit = PerpetualTestkit::new().details_service();

            let failures = testkit.service.refresh(Chain::HyperCore.as_asset_id()).await;

            let steps: Vec<GemPerpetualRefreshStep> = failures.iter().map(|failure| failure.step).collect();
            assert!(steps.contains(&GemPerpetualRefreshStep::Transactions), "a later take profit never arrives without this one");
            let paths = testkit.provider.requested_paths();
            assert!(paths.iter().any(|path| path.contains("devices/transactions") && path.contains("asset_id=hypercore")), "{paths:?}");
        })
    }

    #[test]
    fn test_a_wallet_without_a_hypercore_account_reports_nothing_for_its_positions() {
        block_on(async {
            let testkit = PerpetualTestkit::new().details_service();

            let failures = testkit.service.refresh(Chain::HyperCore.as_asset_id()).await;

            assert!(!failures.iter().any(|failure| failure.step == GemPerpetualRefreshStep::Positions), "{failures:?}");
        })
    }

    #[test]
    fn test_without_a_current_wallet_only_the_transactions_fail_for_that_reason() {
        block_on(async {
            let testkit = PerpetualTestkit::new().details_service();
            testkit.session.set_current_wallet_id(None).unwrap();

            let failures = testkit.service.refresh(Chain::HyperCore.as_asset_id()).await;

            let transactions = failures.iter().find(|failure| failure.step == GemPerpetualRefreshStep::Transactions).unwrap();
            assert!(transactions.message.contains("no current wallet"), "{transactions:?}");
        })
    }
}
