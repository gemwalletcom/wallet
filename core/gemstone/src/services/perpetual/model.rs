use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::services::failures::StepFailure;
use primitives::chart::ChartCandleUpdate;
use primitives::{Asset, PerpetualAccountMode, PerpetualDirection, PerpetualMarginType, PerpetualProvider};
use serde::{Deserialize, Serialize};

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
    pub usdc_value: GemBigInt,
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

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemMarketsRefreshTrigger {
    Scheduled,
    UserRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPerpetualRefreshStep {
    Positions,
    Markets,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemPerpetualRefreshFailure {
    pub step: GemPerpetualRefreshStep,
    pub message: String,
}

impl StepFailure for GemPerpetualRefreshFailure {
    type Step = GemPerpetualRefreshStep;

    fn new(step: GemPerpetualRefreshStep, message: String) -> Self {
        Self { step, message }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum GemPerpetualPositionKind {
    Open { direction: PerpetualDirection },
    Increase,
    Reduce,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct GemPerpetualTransferData {
    pub provider: PerpetualProvider,
    pub direction: PerpetualDirection,
    pub asset: Asset,
    pub base_asset: Asset,
    pub asset_index: i32,
    pub price: f64,
    pub leverage: u8,
    pub margin_type: PerpetualMarginType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, uniffi::Enum)]
pub enum GemPerpetualPositionAction {
    Open {
        data: GemPerpetualTransferData,
    },
    Increase {
        data: GemPerpetualTransferData,
    },
    Reduce {
        data: GemPerpetualTransferData,
        #[serde(with = "crate::models::custom_types::decimal_string::unsigned")]
        available: GemBigUint,
    },
}

#[uniffi::export]
impl GemPerpetualPositionAction {
    pub fn transfer_data(&self) -> GemPerpetualTransferData {
        self.data().clone()
    }
}

impl GemPerpetualPositionAction {
    pub fn data(&self) -> &GemPerpetualTransferData {
        match self {
            Self::Open { data } | Self::Increase { data } | Self::Reduce { data, .. } => data,
        }
    }
}
