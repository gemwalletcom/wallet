use primitives::{chart::ChartCandleStick, perpetual::PerpetualAccountMode};

pub type GemPerpetualAccountMode = PerpetualAccountMode;
pub type GemChartCandleStick = ChartCandleStick;

#[derive(Debug, Clone, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum GemPerpetualSubscription {
    AccountState { address: String },
    SpotState { address: String },
    OpenOrders { address: String },
    Candle { symbol: String, interval: String },
    MarketData { symbol: String },
    MarketPrices,
}
