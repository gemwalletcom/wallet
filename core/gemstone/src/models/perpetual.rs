use primitives::{chart::ChartCandleStick, perpetual::PerpetualAccountMode};

pub type GemPerpetualAccountMode = PerpetualAccountMode;
pub type GemChartCandleStick = ChartCandleStick;

#[derive(Debug, Clone, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum GemPerpetualSubscription {
    Candle { symbol: String, interval: String },
    MarketData { symbol: String },
    MarketPrices,
}
