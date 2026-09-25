use super::rules;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::custom_types::GemBigInt;
use crate::models::list::{GemListRow, GemListSection};
use crate::services::assets::model::{GemAssetItemRow, GemAssetItemTrailing, GemHeaderActions, GemPriceRow, GemRowText};
use crate::services::failures::StepFailure;
use crate::services::localization::GemLocalizedText;
use primitives::chart::{ChartCandleStick, ChartCandleUpdate};
use primitives::perpetual::{PerpetualBalance, PerpetualData, PerpetualPositionData};
use primitives::{Asset, AssetId, PerpetualAccountMode, PerpetualDirection, PerpetualMarginType, PerpetualPosition, PerpetualProvider, PerpetualType, WalletType};
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
    pub id: String,
    pub asset_id: AssetId,
    pub row: GemAssetItemRow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerpetualPositionLine {
    pub id: String,
    pub asset_id: AssetId,
    pub icon: crate::services::assets::icon::GemAssetIcon,
    pub title: String,
    pub position: GemLocalizedText,
    pub direction_tone: GemValueTone,
    pub margin: GemFormattedNumber,
    pub pnl: GemLocalizedText,
    pub pnl_tone: GemValueTone,
}

impl From<PerpetualPositionLine> for GemPerpetualPositionRow {
    fn from(line: PerpetualPositionLine) -> Self {
        Self {
            id: line.id,
            asset_id: line.asset_id,
            row: GemAssetItemRow {
                icon: line.icon,
                title: line.title,
                title_extra: None,
                subtitle: Some(GemRowText {
                    text: line.position,
                    tone: line.direction_tone,
                }),
                subtitle_extra: None,
                trailing: GemAssetItemTrailing::Value {
                    value: GemRowText::number(line.margin),
                    extra: Some(GemRowText { text: line.pnl, tone: line.pnl_tone }),
                },
                masks_balance: true,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerpetualOpenLine {
    pub position: GemLocalizedText,
    pub direction_tone: GemValueTone,
    pub size: Option<GemFormattedNumber>,
}

#[uniffi::export]
pub fn perpetual_open_row(asset_id: AssetId, title: String, direction: PerpetualDirection, leverage: u8, size: f64) -> GemAssetItemRow {
    let line = rules::open_row(direction, leverage, size);
    GemAssetItemRow {
        icon: crate::services::assets::icon::asset_icon(&asset_id),
        title,
        title_extra: None,
        subtitle: Some(GemRowText {
            text: line.position,
            tone: line.direction_tone,
        }),
        subtitle_extra: None,
        trailing: line.size.map_or(GemAssetItemTrailing::None, |size| GemAssetItemTrailing::Value {
            value: GemRowText::number(size),
            extra: None,
        }),
        masks_balance: false,
    }
}

#[uniffi::export]
pub fn perpetual_position_rows(positions: Vec<PerpetualPositionData>) -> Vec<GemPerpetualPositionRow> {
    positions.iter().map(|data| rules::position_row(&data.perpetual, &data.asset, &data.position)).collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerpetualMarketLine {
    pub asset_id: AssetId,
    pub icon: crate::services::assets::icon::GemAssetIcon,
    pub title: String,
    pub price: GemPriceRow,
    pub volume_24h: GemFormattedNumber,
    pub open_interest: GemFormattedNumber,
    pub funding_apr: GemFormattedNumber,
}

impl PerpetualMarketLine {
    pub fn item_row(self) -> GemAssetItemRow {
        let price = self.price.price;
        GemAssetItemRow {
            icon: self.icon,
            title: self.title,
            title_extra: None,
            subtitle_extra: self.price.change.filter(|_| price.is_some()).map(GemRowText::number),
            subtitle: price.map(|price| GemRowText::neutral(GemLocalizedText::Number { number: price })),
            trailing: GemAssetItemTrailing::Value {
                value: GemRowText::number(self.volume_24h),
                extra: None,
            },
            masks_balance: false,
        }
    }
}

#[uniffi::export]
pub fn perpetual_market_rows(markets: Vec<PerpetualData>) -> Vec<GemAssetItemRow> {
    markets.iter().map(|data| rules::market_row(&data.perpetual, &data.asset).item_row()).collect()
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPerpetualMarketItem {
    pub data: PerpetualData,
    pub row: GemAssetItemRow,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPerpetualMarketSections {
    pub pinned: Vec<GemPerpetualMarketItem>,
    pub markets: Vec<GemPerpetualMarketItem>,
}

#[uniffi::export]
pub fn perpetual_market_sections(markets: Vec<PerpetualData>) -> GemPerpetualMarketSections {
    let (pinned, markets): (Vec<_>, Vec<_>) = markets
        .into_iter()
        .map(|data| GemPerpetualMarketItem {
            row: rules::market_row(&data.perpetual, &data.asset).item_row(),
            data,
        })
        .partition(|item| item.data.metadata.is_pinned);
    GemPerpetualMarketSections { pinned, markets }
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
    pub tones: Vec<GemValueTone>,
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
pub enum GemPerpetualEnablementTrigger {
    Foreground,
    WalletChanged,
    PreferenceChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPerpetualRefreshStep {
    Positions,
    Markets,
    Transactions,
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
    Info { buttons: Vec<GemPerpetualButtonRow>, rows: Vec<GemListRow> },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualButtonRow {
    pub button: GemPerpetualButton,
    pub tone: GemValueTone,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualDetails {
    pub title: String,
    pub sections: Vec<GemPerpetualSection>,
    pub modify_buttons: Vec<GemPerpetualButtonRow>,
    pub position: Option<PerpetualPosition>,
    pub position_row: Option<GemPerpetualPositionRow>,
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

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemPerpetualMarketQuery {
    pub search: String,
    pub limit: u32,
    pub requires_volume: bool,
}

#[uniffi::export]
pub fn perpetual_market_query(search: String) -> GemPerpetualMarketQuery {
    super::rules::market_query(search.trim().to_string())
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
    fn test_market_sections_split_pinned_markets_and_keep_their_order() {
        let market = |name: &str, is_pinned: bool| PerpetualData {
            perpetual: primitives::Perpetual {
                name: name.to_string(),
                ..primitives::Perpetual::mock()
            },
            asset: primitives::Asset::mock(),
            metadata: primitives::PerpetualMetadata { is_pinned },
        };

        let sections = perpetual_market_sections(vec![market("A", false), market("B", true), market("C", false)]);

        assert_eq!(sections.pinned.iter().map(|item| item.data.perpetual.name.as_str()).collect::<Vec<_>>(), vec!["B"]);
        assert_eq!(sections.markets.iter().map(|item| item.data.perpetual.name.as_str()).collect::<Vec<_>>(), vec!["A", "C"]);
    }

    #[test]
    fn test_browsing_lists_traded_markets_and_a_search_reaches_every_one() {
        let browsing = perpetual_market_query(String::new());
        assert!(browsing.requires_volume, "the market list is what is being traded");
        assert_eq!(browsing.limit, super::super::rules::MARKETS_LIMIT);

        let searching = perpetual_market_query("  btc ".to_string());
        assert_eq!(searching.search, "btc", "the query reaches the store trimmed");
        assert!(!searching.requires_volume, "a search reaches a market that has not traded today");
        assert_eq!(searching.limit, browsing.limit);
    }

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
    pub deposit_asset: Asset,
    pub withdraw_asset: Asset,
}

#[uniffi::export]
pub fn perpetual_balance_total(balance: Option<PerpetualBalance>) -> GemFormattedNumber {
    rules::balance_total(balance.as_ref())
}

#[uniffi::export]
pub fn perpetual_balance_header(balance: Option<PerpetualBalance>, wallet_type: WalletType) -> GemPerpetualBalanceHeader {
    rules::balance_header(balance, wallet_type)
}
