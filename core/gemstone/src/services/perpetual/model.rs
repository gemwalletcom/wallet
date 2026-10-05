use super::rules;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::custom_types::GemBigInt;
use crate::models::list::{GemListRow, GemListSection};
use crate::models::state::GemListPhase;
use crate::services::assets::model::{GemAssetItemRow, GemAssetItemTrailing, GemPriceRow, GemRowText, GemValueHeader};
use crate::services::chart::candlestick_header;
use crate::services::chart::model::{GemChartDateStyle, GemChartHeader, GemChartSelection};
use crate::services::chart::rules as chart_rules;
use crate::services::failures::StepFailure;
use crate::services::localization::GemLocalizedText;
use chrono::{DateTime, Utc};
use primitives::chart::{ChartCandleStick, ChartCandleUpdate};
use primitives::perpetual::{PerpetualBalance, PerpetualData, PerpetualPositionData};
use primitives::{Asset, AssetId, PerpetualAccountMode, PerpetualDirection, PerpetualId, PerpetualMarginType, PerpetualPosition, PerpetualProvider, PerpetualType, WalletType};
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
            subtitle: price.map(GemRowText::neutral_number),
            trailing: GemAssetItemTrailing::Value {
                value: GemRowText::number(self.volume_24h),
                extra: None,
            },
            masks_balance: false,
        }
    }
}

pub fn perpetual_market_items(markets: Vec<PerpetualData>) -> Vec<GemPerpetualMarketItem> {
    markets
        .into_iter()
        .map(|data| GemPerpetualMarketItem {
            row: rules::market_row(&data.perpetual, &data.asset).item_row(),
            data,
        })
        .collect()
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
    let (pinned, markets): (Vec<_>, Vec<_>) = perpetual_market_items(markets).into_iter().partition(|item| item.data.metadata.is_pinned);
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
    pub label: GemLocalizedText,
    pub overlap_level: u32,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualChartLayout {
    pub price_low: f64,
    pub price_high: f64,
    pub levels: Vec<GemFormattedNumber>,
    pub lines: Vec<GemPerpetualChartLine>,
    pub current_price: GemFormattedNumber,
    pub current_tone: GemValueTone,
    pub tones: Vec<GemValueTone>,
    pub volume_high: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemCandleTickFormat {
    Time,
    Day,
    MonthYear,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCandleTick {
    pub date: DateTime<Utc>,
    pub format: GemCandleTickFormat,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCandleChart {
    pub candles: Vec<ChartCandleStick>,
    pub layout: GemPerpetualChartLayout,
    pub header: GemChartHeader,
    pub date_style: GemChartDateStyle,
    pub base: f64,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub body_width: f64,
    pub x_ticks: Vec<GemCandleTick>,
    pub is_zoomed: bool,
}

#[uniffi::export]
impl GemCandleChart {
    pub fn selection(&self, index: u32) -> Option<GemChartSelection> {
        let candle = self.candles.get(index as usize)?;
        Some(GemChartSelection {
            header: candlestick_header(self.base, candle.close),
            date: candle.date,
        })
    }

    pub fn tooltip(&self, index: u32) -> Option<GemCandleTooltip> {
        self.candles.get(index as usize).map(rules::candle_tooltip)
    }

    pub fn index_at(&self, fraction: f64) -> Option<u32> {
        chart_rules::nearest_index(self.candles.iter().map(|candle| candle.date), self.start..=self.end, fraction)
    }
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
    Header,
    Positions,
    Recents,
    Pinned,
    Markets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemPerpetualMarketCounts {
    pub positions: usize,
    pub pinned: usize,
    pub markets: usize,
    pub recents: usize,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualMarketView {
    pub sections: Vec<GemPerpetualMarketSection>,
    pub phase: GemListPhase,
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

    pub fn view(&self, position_ids: Vec<String>, pinned_ids: Vec<PerpetualId>, market_ids: Vec<PerpetualId>, recent_asset_ids: Vec<AssetId>) -> GemPerpetualMarketView {
        let counts = GemPerpetualMarketCounts {
            positions: position_ids.len(),
            pinned: pinned_ids.len(),
            markets: market_ids.len(),
            recents: recent_asset_ids.len(),
        };
        super::rules::market_view(&counts, self.is_searching, self.search_query().is_empty())
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

    #[test]
    fn test_the_wallet_preview_offers_trading_until_a_position_is_open() {
        assert_eq!(perpetual_preview(vec![], None), GemPerpetualPreview::Trade { balance: GemFormattedNumber::usd(0.0) });
        assert_eq!(perpetual_preview(vec!["position".to_string()], None), GemPerpetualPreview::Positions);
    }
}

#[uniffi::export]
pub fn perpetual_preview(position_ids: Vec<String>, balance: Option<PerpetualBalance>) -> GemPerpetualPreview {
    match position_ids.is_empty() {
        true => GemPerpetualPreview::Trade {
            balance: rules::balance_total(balance.as_ref()),
        },
        false => GemPerpetualPreview::Positions,
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemPerpetualPreview {
    Trade { balance: GemFormattedNumber },
    Positions,
}

#[uniffi::export]
pub fn perpetual_balance_header(balance: Option<PerpetualBalance>, wallet_type: WalletType) -> GemValueHeader {
    rules::balance_header(balance, wallet_type)
}
