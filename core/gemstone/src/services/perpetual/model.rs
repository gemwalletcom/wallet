use primitives::chart::ChartCandleUpdate;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemPerpetualSocketUpdate {
    Applied,
    Candle { candle: ChartCandleUpdate },
    SubscriptionResponse { subscription_type: String },
    Error { message: String },
    Unknown,
}
