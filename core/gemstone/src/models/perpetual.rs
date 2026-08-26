use std::collections::HashMap;

use gem_hypercore::models::order::OpenOrder;
use gem_hypercore::models::websocket::{HyperliquidSocketMessage, PositionsDiff};
use primitives::{
    AssetId, PerpetualId, PerpetualMarginType, PerpetualMarketData, PerpetualOrderType, PerpetualPosition, PerpetualProvider, PerpetualTriggerOrder,
    chart::{ChartCandleStick, ChartCandleUpdate, ChartDateValue},
    perpetual::{Perpetual, PerpetualAccountMode, PerpetualBalance, PerpetualData, PerpetualMetadata, PerpetualPositionsSummary},
};

pub type GemHyperliquidOpenOrder = OpenOrder;
pub type GemPositionsDiff = PositionsDiff;
pub type GemPerpetualMarginType = PerpetualMarginType;
pub type GemPerpetualOrderType = PerpetualOrderType;
pub type GemPerpetualPositionsSummary = PerpetualPositionsSummary;
pub type GemPerpetualBalance = PerpetualBalance;
pub type GemPerpetualAccountMode = PerpetualAccountMode;
pub type GemPerpetualPosition = PerpetualPosition;
pub type GemPerpetual = Perpetual;
pub type GemPerpetualMetadata = PerpetualMetadata;
pub type GemChartCandleStick = ChartCandleStick;
pub type GemChartCandleUpdate = ChartCandleUpdate;
pub type GemChartDateValue = ChartDateValue;
pub type GemPerpetualData = PerpetualData;
pub type GemPerpetualMarketData = PerpetualMarketData;

pub type GemPerpetualTriggerOrder = PerpetualTriggerOrder;

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

#[uniffi::remote(Record)]
pub struct GemPerpetual {
    pub id: PerpetualId,
    pub name: String,
    pub provider: PerpetualProvider,
    pub asset_id: AssetId,
    pub identifier: String,
    pub price: f64,
    pub price_percent_change_24h: f64,
    pub open_interest: f64,
    pub volume_24h: f64,
    pub funding: f64,
    pub max_leverage: u8,
    pub is_isolated_only: bool,
}
