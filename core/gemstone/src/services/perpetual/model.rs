use super::rules;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::custom_types::GemBigInt;
use crate::models::list::{GemListRow, GemListSection};
use crate::services::assets::model::GemHeaderActions;
use crate::services::failures::StepFailure;
use crate::services::localization::GemLocalizedText;
use primitives::chart::{ChartCandleStick, ChartCandleUpdate};
use primitives::perpetual::PerpetualBalance;
use primitives::{Asset, Perpetual, PerpetualAccountMode, PerpetualDirection, PerpetualMarginType, PerpetualPosition, PerpetualProvider, PerpetualType, WalletType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum GemPerpetualSocketUpdate {
    Applied,
    Candle { candle: ChartCandleUpdate },
    SubscriptionResponse { subscription_type: String },
    Error { message: String },
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GemPerpetualOrderAction {
    Open,
    Increase,
    Reduce { position_direction: PerpetualDirection },
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
pub struct GemPerpetualConfirmDetailsSummary {
    pub text: Option<GemLocalizedText>,
    pub tone: GemValueTone,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualConfirmDetails {
    pub id: String,
    pub summary: GemPerpetualConfirmDetailsSummary,
    pub sections: Vec<GemListSection>,
}

#[uniffi::export]
pub fn perpetual_confirm_details(perpetual_type: PerpetualType) -> Option<GemPerpetualConfirmDetails> {
    rules::confirm_details(&perpetual_type)
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualPositionRow {
    pub title: String,
    pub leverage: String,
    pub direction: PerpetualDirection,
}

#[uniffi::export]
pub fn perpetual_position_row(perpetual: Perpetual, asset: Asset, position: PerpetualPosition) -> GemPerpetualPositionRow {
    rules::position_row(&perpetual, &asset, &position)
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualMarketRow {
    pub title: String,
    pub shows_price: bool,
    pub volume_24h: GemFormattedNumber,
    pub open_interest: GemFormattedNumber,
    pub funding_apr: GemFormattedNumber,
}

#[uniffi::export]
pub fn perpetual_market_row(perpetual: Perpetual, asset: Asset) -> GemPerpetualMarketRow {
    rules::market_row(&perpetual, &asset)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPerpetualChartLineKind {
    Entry,
    TakeProfit,
    StopLoss,
    Liquidation,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualChartLine {
    pub kind: GemPerpetualChartLineKind,
    pub price: GemFormattedNumber,
    pub overlap_level: u32,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualChartLayout {
    pub price_low: f64,
    pub price_high: f64,
    pub ticks: Vec<GemFormattedNumber>,
    pub x_tick_count: u32,
    pub lines: Vec<GemPerpetualChartLine>,
    pub current_price: Option<GemFormattedNumber>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemCandleTooltipRow {
    Open,
    High,
    Low,
    Close,
    Change,
    Volume,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCandleTooltipCell {
    pub row: GemCandleTooltipRow,
    pub value: GemFormattedNumber,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCandleTooltip {
    pub prices: Vec<GemCandleTooltipCell>,
    pub summary: Vec<GemCandleTooltipCell>,
}

#[uniffi::export]
pub fn candle_tooltip(candle: ChartCandleStick) -> GemCandleTooltip {
    rules::candle_tooltip(&candle)
}

#[uniffi::export]
pub fn perpetual_chart_layout(candles: Vec<ChartCandleStick>, position: Option<PerpetualPosition>) -> GemPerpetualChartLayout {
    rules::chart_layout(&candles, position.as_ref())
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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemPerpetualSection {
    Position { rows: Vec<GemPerpetualPositionDetail> },
    Info { buttons: Vec<GemPerpetualButton>, rows: Vec<GemListRow> },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualDetails {
    pub title: String,
    pub sections: Vec<GemPerpetualSection>,
    pub modify_buttons: Vec<GemPerpetualButton>,
    pub position: Option<PerpetualPosition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPerpetualPositionDetailRow {
    Pnl,
    Autoclose,
    Size,
    EntryPrice,
    LiquidationPrice,
    Margin,
    FundingPayments,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualPositionDetail {
    pub kind: GemPerpetualPositionDetailRow,
    pub row: GemListRow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPerpetualButton {
    Long,
    Short,
    Modify,
    Close,
    Increase,
    Reduce,
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
#[allow(clippy::large_enum_variant)]
pub enum GemPerpetualPositionAction {
    Open { data: GemPerpetualTransferData },
    Increase { data: GemPerpetualTransferData },
    Reduce { data: GemPerpetualTransferData, position: PerpetualPosition },
}

#[uniffi::export]
impl GemPerpetualPositionAction {
    pub fn transfer_data(&self) -> GemPerpetualTransferData {
        self.data().clone()
    }

    pub fn shows_autoclose(&self) -> bool {
        matches!(self, Self::Open { .. })
    }
}

impl GemPerpetualPositionAction {
    pub fn data(&self) -> &GemPerpetualTransferData {
        match self {
            Self::Open { data } | Self::Increase { data } | Self::Reduce { data, .. } => data,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPerpetualMarketSection {
    Positions,
    Recents,
    Pinned,
    Markets,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemPerpetualMarketCounts {
    pub positions: u32,
    pub pinned: u32,
    pub markets: u32,
    pub recents: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, uniffi::Record)]
pub struct GemPerpetualMarketSession {
    pub query: String,
    pub is_searching: bool,
}

#[uniffi::export]
impl GemPerpetualMarketSession {
    pub fn on_query_changed(&self, query: String) -> Self {
        Self { query, ..self.clone() }
    }

    pub fn on_searching_changed(&self, is_searching: bool) -> Self {
        Self { is_searching, ..self.clone() }
    }

    pub fn search_query(&self) -> String {
        self.query.trim().to_string()
    }

    pub fn sections(&self, counts: GemPerpetualMarketCounts) -> Vec<GemPerpetualMarketSection> {
        super::rules::market_sections(&counts, self.is_searching, self.search_query().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_opening_a_position_shows_autoclose() {
        let data = GemPerpetualTransferData::mock();

        assert!(GemPerpetualPositionAction::Open { data: data.clone() }.shows_autoclose());
        assert!(!GemPerpetualPositionAction::Increase { data }.shows_autoclose());
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualBalanceHeader {
    pub total: GemFormattedNumber,
    pub available: GemFormattedNumber,
    pub actions: GemHeaderActions,
}

#[uniffi::export]
pub fn perpetual_balance_total(balance: Option<PerpetualBalance>) -> GemFormattedNumber {
    rules::balance_total(balance.as_ref())
}

#[uniffi::export]
pub fn perpetual_balance_header(balance: Option<PerpetualBalance>, wallet_type: WalletType) -> GemPerpetualBalanceHeader {
    rules::balance_header(balance, wallet_type)
}
