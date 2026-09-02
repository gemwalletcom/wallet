use std::sync::Arc;

use primitives::{AssetId, Chain, ChartPeriod, Currency, WalletId};

use super::GemPerpetualService;
use crate::models::perpetual::{GemChartCandleStick, GemPerpetualSubscription};
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::transactions::GemTransactionsService;

#[derive(uniffi::Object)]
pub struct GemPerpetualDetailsService {
    perpetuals: Arc<GemPerpetualService>,
    transactions: Arc<GemTransactionsService>,
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemPerpetualDetailsService {
    #[uniffi::constructor]
    pub fn new(perpetuals: Arc<GemPerpetualService>, transactions: Arc<GemTransactionsService>, preferences: Arc<GemPreferencesService>) -> Self {
        Self {
            perpetuals,
            transactions,
            preferences,
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

    pub async fn sync_transactions(&self, wallet_id: WalletId, asset_id: AssetId) -> Result<(), GemServiceError> {
        self.transactions.sync(wallet_id, Some(asset_id)).await
    }
}
