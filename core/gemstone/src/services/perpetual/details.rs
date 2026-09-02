use std::sync::Arc;

use primitives::{AssetId, Chain, ChartPeriod, Currency};

use super::GemPerpetualService;
use crate::models::perpetual::{GemChartCandleStick, GemPerpetualSubscription};
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::transactions::GemTransactionsService;
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

    pub fn currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn chart_period(&self) -> ChartPeriod {
        self.preferences.get_perpetual_chart_period()
    }

    pub fn set_chart_period(&self, period: ChartPeriod) -> Result<(), GemServiceError> {
        self.preferences.set_perpetual_chart_period(period)
    }

    pub fn candle_interval(&self, period: ChartPeriod) -> String {
        self.perpetuals.candle_interval(period)
    }

    pub fn candle_subscription(&self, symbol: String, period: ChartPeriod) -> GemPerpetualSubscription {
        GemPerpetualSubscription::Candle {
            symbol,
            interval: self.perpetuals.candle_interval(period),
        }
    }

    pub async fn candlesticks(&self, symbol: String, period: ChartPeriod) -> Result<Vec<GemChartCandleStick>, GemServiceError> {
        self.perpetuals.get_candlesticks(Chain::HyperCore, symbol, period).await
    }

    pub fn merge_candle(&self, candles: Vec<GemChartCandleStick>, candle: GemChartCandleStick) -> Vec<GemChartCandleStick> {
        self.perpetuals.merge_candle(candles, candle)
    }

    pub async fn sync_transactions(&self, asset_id: AssetId) -> Result<(), GemServiceError> {
        self.transactions.sync(self.session.current_wallet_id()?, Some(asset_id)).await
    }
}
