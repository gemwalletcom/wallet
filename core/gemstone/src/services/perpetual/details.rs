use std::sync::Arc;

use primitives::chart::ChartCandleUpdate;
use primitives::{Asset, AssetId, Chain, ChartPeriod, Currency, Perpetual, PerpetualPosition};

use super::model::{GemPerpetualPositionAction, GemPerpetualPositionKind};
use super::{GemPerpetualService, rules};
use crate::models::perpetual::{GemChartCandleStick, GemPerpetualSubscription};
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::transactions::GemTransactionsService;
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
    pub fn new(
        perpetuals: Arc<GemPerpetualService>,
        transactions: Arc<GemTransactionsService>,
        preferences: Arc<GemPreferencesService>,
        session: Arc<GemWalletSessionService>,
    ) -> Self {
        Self {
            perpetuals,
            transactions,
            preferences,
            session,
        }
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn position_action(
        &self,
        perpetual: Perpetual,
        asset: Asset,
        position: Option<PerpetualPosition>,
        kind: GemPerpetualPositionKind,
    ) -> Result<GemPerpetualPositionAction, GemServiceError> {
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
        GemPerpetualSubscription::MarketData {
            symbol: rules::symbol(&perpetual),
        }
    }

    pub async fn candlesticks(&self, perpetual: Perpetual, period: ChartPeriod) -> Result<Vec<GemChartCandleStick>, GemServiceError> {
        self.perpetuals.get_candlesticks(Chain::HyperCore, rules::symbol(&perpetual), period).await
    }

    pub fn apply_candle_update(&self, candles: Vec<GemChartCandleStick>, update: ChartCandleUpdate, perpetual: Perpetual, period: ChartPeriod) -> Option<Vec<GemChartCandleStick>> {
        rules::apply_candle_update(candles, update, &perpetual, &period)
    }

    pub async fn sync_positions(&self) -> Result<(), GemServiceError> {
        self.perpetuals.sync_current_positions().await
    }

    pub async fn sync_transactions(&self, asset_id: AssetId) -> Result<(), GemServiceError> {
        self.transactions.sync_wallet(self.session.current_wallet_id()?, Some(asset_id)).await
    }
}
