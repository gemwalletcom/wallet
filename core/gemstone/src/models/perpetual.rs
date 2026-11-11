#[derive(Debug, Clone, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum GemPerpetualSubscription {
    Candle { symbol: String, interval: String },
    MarketData { symbol: String },
    MarketPrices,
}
