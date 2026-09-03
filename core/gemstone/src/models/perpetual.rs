use std::collections::HashMap;

use gem_hypercore::models::order::OpenOrder;
use gem_hypercore::models::websocket::{HyperliquidSocketMessage, PositionsDiff};
use primitives::{
    PerpetualMarketData, PerpetualPosition,
    chart::{ChartCandleStick, ChartCandleUpdate},
    perpetual::{PerpetualAccountMode, PerpetualBalance},
};

pub type GemHyperliquidOpenOrder = OpenOrder;
pub type GemPositionsDiff = PositionsDiff;
pub type GemPerpetualAccountMode = PerpetualAccountMode;
pub type GemChartCandleStick = ChartCandleStick;
pub type GemPerpetualMarketData = PerpetualMarketData;

#[uniffi::remote(Record)]
pub struct GemPositionsDiff {
    pub delete_position_ids: Vec<String>,
    pub positions: Vec<PerpetualPosition>,
}

#[uniffi::remote(Record)]
pub struct GemHyperliquidOpenOrder {
    pub coin: String,
    pub oid: u64,
    pub trigger_px: Option<f64>,
    pub limit_px: Option<f64>,
    pub is_position_tpsl: bool,
    pub order_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum GemPerpetualSubscription {
    AccountState { address: String },
    SpotState { address: String },
    OpenOrders { address: String },
    Candle { symbol: String, interval: String },
    MarketData { symbol: String },
    MarketPrices,
}

pub type GemHyperliquidSocketMessage = HyperliquidSocketMessage;

#[uniffi::remote(Enum)]
pub enum GemHyperliquidSocketMessage {
    AccountState {
        balance: Option<PerpetualBalance>,
        positions: Vec<PerpetualPosition>,
    },
    SpotState {
        balance: PerpetualBalance,
    },
    OpenOrders {
        orders: Vec<GemHyperliquidOpenOrder>,
    },
    Candle {
        candle: ChartCandleUpdate,
    },
    MarketData {
        market: GemPerpetualMarketData,
    },
    MarketPrices {
        prices: HashMap<String, f64>,
    },
    SubscriptionResponse {
        subscription_type: String,
    },
    Error {
        message: String,
    },
    Unknown,
}
