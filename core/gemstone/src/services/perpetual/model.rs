use primitives::chart::ChartCandleUpdate;
use primitives::{Asset, PerpetualAccountMode, PerpetualDirection, PerpetualMarginType};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemPerpetualSocketUpdate {
    Applied,
    Candle { candle: ChartCandleUpdate },
    SubscriptionResponse { subscription_type: String },
    Error { message: String },
    Unknown,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemPerpetualOrderAction {
    Open,
    Increase,
    Reduce { position_direction: PerpetualDirection },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualOrderInput {
    pub action: GemPerpetualOrderAction,
    pub direction: PerpetualDirection,
    pub margin_type: PerpetualMarginType,
    pub base_asset: Asset,
    pub asset: Asset,
    pub asset_index: i32,
    pub price: f64,
    pub usdc_amount: String,
    pub usdc_decimals: i32,
    pub leverage: u8,
    pub slippage: Option<f64>,
    pub take_profit: Option<String>,
    pub stop_loss: Option<String>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualCloseInput {
    pub asset_index: i32,
    pub direction: PerpetualDirection,
    pub margin_type: PerpetualMarginType,
    pub base_asset: Asset,
    pub asset: Asset,
    pub market_price: f64,
    pub size: f64,
    pub leverage: u8,
    pub pnl: f64,
    pub entry_price: f64,
    pub margin_amount: f64,
    pub slippage: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualConnection {
    pub address: String,
    pub mode: PerpetualAccountMode,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAutocloseSummary {
    pub take_profit: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit_cleared: bool,
    pub stop_loss_cleared: bool,
}
