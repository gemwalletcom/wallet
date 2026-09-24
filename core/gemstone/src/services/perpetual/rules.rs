use crate::percentage::GemPercentageStyle;
use crate::precision::GemCurrencyStyle;
use chrono::Utc;
use number_formatter::{BigNumberFormatter, NumberFormatterError};
use primitives::PriceChangeCalculator;
use primitives::chart::{ChartCandleStick, ChartCandleUpdate};
use primitives::currency::Currency;
use primitives::known_assets::HYPERCORE_PERPETUAL_USDC;
use primitives::perpetual::{PerpetualBalance, PerpetualData, PerpetualMarketData};
use primitives::{
    Asset, AssetBasic, AssetId, AssetPrice, AssetProperties, AssetScore, AssetType, Chain, ChartPeriod, Perpetual, PerpetualAccountMode, PerpetualDirection, PerpetualMarginType, PerpetualPosition, PerpetualProvider, WalletType,
};
use strum::IntoEnumIterator;

use super::model::{
    GemCandleTooltip, GemCandleTooltipCell, GemCandleTooltipRow, GemMarketsRefreshTrigger, GemPerpetualBalanceHeader, GemPerpetualButton, GemPerpetualButtonRow, GemPerpetualChartLayout, GemPerpetualChartLine, GemPerpetualChartLineKind,
    GemPerpetualCloseInput, GemPerpetualConfirmDetails, GemPerpetualConfirmDetailsSummary, GemPerpetualDetails, GemPerpetualMarketCounts, GemPerpetualMarketQuery, GemPerpetualMarketRow, GemPerpetualMarketSection, GemPerpetualOpenRow,
    GemPerpetualOrderAction, GemPerpetualOrderInput, GemPerpetualPositionAction, GemPerpetualPositionDetail, GemPerpetualPositionDetailRow, GemPerpetualPositionKind, GemPerpetualPositionRow, GemPerpetualSection, GemPerpetualTransferData,
};
use crate::formatted_number::{GemFormattedNumber, GemValueTone, value_tone};
use crate::models::custom_types::GemBigInt;
use crate::models::list::{GemInfoTopic, GemListRow, GemListRowTitle, GemListSection, GemListSectionFooter, GemListSectionTitle};
use crate::models::placeholder::EMPTY_VALUE;
use crate::perpetual::GemPerpetual;
use crate::services::assets::model::{GemHeaderActions, GemHeaderButton, GemHeaderButtonKind};
use crate::services::error::GemServiceError;
use crate::services::localization::{GemLocalizedText, GemPositionChange, GemTriggerOrder};
use crate::services::transfer::GemTransferData;
use num_bigint::BigUint;
use primitives::{PerpetualConfirmData, PerpetualModifyConfirmData, PerpetualModifyPositionType, PerpetualReduceData, PerpetualType};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::models::asset::wallet_default_assets;
use crate::services::balance::{GemBalanceUpdate, GemBalanceUpdateType};
use crate::services::collections::stale;

pub use gem_hypercore::provider::perpetual::candle_interval;

const MARKETS_REFRESH_INTERVAL_SECONDS: i64 = 60 * 60;

const DEFAULT_SLIPPAGE_PERCENT: f64 = 2.0;
const HOURS_PER_YEAR: f64 = 24.0 * 365.0;

const CHART_RANGE_PADDING_FRACTION: f64 = 0.05;
const CHART_RANGE_FLOOR_FRACTION: f64 = 0.95;
const CHART_LINE_VISIBILITY_BUFFER_FRACTION: f64 = 0.5;
const CHART_LABEL_OVERLAP_FRACTION: f64 = 0.06;
const CHART_MINIMUM_SPAN_FRACTION: f64 = 0.001;
const CHART_MINIMUM_SPAN: f64 = 1e-9;
const CHART_TICK_COUNT: usize = 4;
const CHART_X_TICK_COUNT: usize = 6;

pub fn perpetual_asset_basics(data: &[PerpetualData]) -> Vec<AssetBasic> {
    data.iter()
        .map(|item| {
            AssetBasic::new(
                item.asset.clone(),
                AssetProperties {
                    is_enabled: false,
                    is_buyable: false,
                    is_sellable: false,
                    is_swapable: false,
                    is_stakeable: false,
                    staking_apr: None,
                    is_earnable: false,
                    earn_apr: None,
                    has_image: false,
                    has_price: false,
                },
                AssetScore::new(0),
            )
        })
        .collect()
}

pub fn confirm_details(perpetual_type: &PerpetualType) -> Option<GemPerpetualConfirmDetails> {
    let (direction, data, summarised_by) = match perpetual_type {
        PerpetualType::Open { data } => (data.direction.clone(), data, SummarisedBy::Position),
        PerpetualType::Close { data } => (data.direction.clone(), data, SummarisedBy::Pnl),
        PerpetualType::Increase { data } => (data.direction.clone(), data, SummarisedBy::Change(GemPositionChange::Increase)),
        PerpetualType::Reduce { data } => (data.position_direction.clone(), &data.data, SummarisedBy::Change(GemPositionChange::Reduce)),
        PerpetualType::Modify { .. } => return None,
    };
    let pnl = data.pnl.map(|pnl| pnl_text(pnl, data.margin_amount));
    let summary = match summarised_by {
        SummarisedBy::Position => GemPerpetualConfirmDetailsSummary {
            text: Some(position_text(&direction, data.leverage)),
            tone: direction_tone(&direction),
        },
        SummarisedBy::Pnl => GemPerpetualConfirmDetailsSummary {
            text: pnl.as_ref().map(|(text, _)| text.clone()),
            tone: pnl.as_ref().map_or(GemValueTone::Neutral, |(_, tone)| *tone),
        },
        SummarisedBy::Change(change) => GemPerpetualConfirmDetailsSummary {
            text: Some(GemLocalizedText::PositionChange { change, direction: direction.clone() }),
            tone: GemValueTone::Neutral,
        },
    };
    Some(GemPerpetualConfirmDetails {
        id: data.base_asset.id.to_string(),
        summary,
        sections: details_sections(&direction, data, pnl),
    })
}

enum SummarisedBy {
    Position,
    Pnl,
    Change(GemPositionChange),
}

fn details_sections(direction: &PerpetualDirection, data: &PerpetualConfirmData, pnl: Option<(GemLocalizedText, GemValueTone)>) -> Vec<GemListSection> {
    let label = |title: GemListRowTitle, text: GemLocalizedText, tone: GemValueTone| GemListRow::Label {
        title,
        text,
        tone,
        info: None,
        progress: false,
    };
    let amount = |title: GemListRowTitle, amount: GemFormattedNumber| GemListRow::Amount { title, amount, info: None };

    let mut position_rows = vec![label(GemListRowTitle::Position, position_text(direction, data.leverage), direction_tone(direction))];
    if let Some((text, tone)) = pnl {
        position_rows.push(label(GemListRowTitle::Pnl, text, tone));
    }

    let mut price_rows = vec![amount(GemListRowTitle::MarketPrice, GemFormattedNumber::usd(data.market_price))];
    if let Some(entry_price) = data.entry_price {
        price_rows.push(amount(GemListRowTitle::EntryPrice, GemFormattedNumber::usd(entry_price)));
    }
    price_rows.push(amount(GemListRowTitle::Slippage, GemFormattedNumber::percentage(data.slippage, GemPercentageStyle::Unsigned)));

    [
        Some(position_rows),
        Some(vec![
            amount(GemListRowTitle::Margin, GemFormattedNumber::usd(data.margin_amount)),
            amount(GemListRowTitle::Size, GemFormattedNumber::usd(data.fiat_value)),
        ]),
        trigger_order_section(data),
        Some(price_rows),
    ]
    .into_iter()
    .flatten()
    .map(|rows| GemListSection {
        title: GemListSectionTitle::None,
        footer: GemListSectionFooter::None,
        rows,
    })
    .collect()
}

fn trigger_order_lines(take_profit: TriggerOrderState, stop_loss: TriggerOrderState) -> Vec<GemLocalizedText> {
    let line = |order: GemTriggerOrder, state: TriggerOrderState| {
        (state.price.is_some() || state.is_cleared).then(|| GemLocalizedText::TriggerOrder {
            order,
            price: state.price.map(GemFormattedNumber::usd),
        })
    };
    [line(GemTriggerOrder::TakeProfit, take_profit), line(GemTriggerOrder::StopLoss, stop_loss)].into_iter().flatten().collect()
}

#[derive(Debug, Clone, Copy, Default)]
struct TriggerOrderState {
    price: Option<f64>,
    is_cleared: bool,
}

impl TriggerOrderState {
    fn set(price: Option<f64>) -> Self {
        Self { price, is_cleared: false }
    }
}

pub fn amount_autoclose_row(take_profit: Option<f64>, stop_loss: Option<f64>) -> GemListRow {
    let lines = trigger_order_lines(TriggerOrderState::set(take_profit), TriggerOrderState::set(stop_loss));
    GemListRow::Lines {
        title: GemListRowTitle::AutoClose,
        lines: match lines.is_empty() {
            true => vec![GemLocalizedText::Text { text: EMPTY_VALUE.to_string() }],
            false => lines,
        },
        info: Some(GemInfoTopic::AutoClose),
    }
}

fn trigger_order_section(data: &PerpetualConfirmData) -> Option<Vec<GemListRow>> {
    let price = |value: &Option<String>| TriggerOrderState::set(value.as_ref().and_then(|value| value.parse::<f64>().ok()));
    let lines = trigger_order_lines(price(&data.take_profit), price(&data.stop_loss));
    (!lines.is_empty()).then_some(vec![GemListRow::Lines {
        title: GemListRowTitle::AutoClose,
        lines,
        info: None,
    }])
}

fn position_text(direction: &PerpetualDirection, leverage: u8) -> GemLocalizedText {
    GemLocalizedText::Position {
        direction: direction.clone(),
        leverage: GemFormattedNumber::leverage(leverage as f64),
    }
}

fn direction_tone(direction: &PerpetualDirection) -> GemValueTone {
    match direction {
        PerpetualDirection::Long => GemValueTone::Positive,
        PerpetualDirection::Short => GemValueTone::Negative,
    }
}

fn pnl_text(pnl: f64, margin_amount: f64) -> (GemLocalizedText, GemValueTone) {
    let amount = GemFormattedNumber::signed_usd(pnl);
    let tone = amount.tone;
    (
        GemLocalizedText::Pnl {
            amount,
            percent: GemFormattedNumber::percentage(PriceChangeCalculator::pnl_percentage(pnl, margin_amount), GemPercentageStyle::Signed),
        },
        tone,
    )
}

pub fn autoclose_row(data: &PerpetualModifyConfirmData) -> Option<GemListRow> {
    let orders = data.modify_types.iter().find_map(|modify| match modify {
        PerpetualModifyPositionType::Tpsl { order } => Some(order),
        PerpetualModifyPositionType::Cancel { .. } => None,
    });
    let canceled: HashSet<u64> = data
        .modify_types
        .iter()
        .filter_map(|modify| match modify {
            PerpetualModifyPositionType::Cancel { orders } => Some(orders),
            PerpetualModifyPositionType::Tpsl { .. } => None,
        })
        .flatten()
        .map(|order| order.order_id)
        .collect();

    let take_profit = orders.and_then(|orders| orders.take_profit.as_ref()).and_then(|value| value.parse::<f64>().ok());
    let stop_loss = orders.and_then(|orders| orders.stop_loss.as_ref()).and_then(|value| value.parse::<f64>().ok());
    let take_profit_cleared = take_profit.is_none() && data.take_profit_order_id.is_some_and(|id| canceled.contains(&id));
    let stop_loss_cleared = stop_loss.is_none() && data.stop_loss_order_id.is_some_and(|id| canceled.contains(&id));

    let lines = trigger_order_lines(
        TriggerOrderState {
            price: take_profit,
            is_cleared: take_profit_cleared,
        },
        TriggerOrderState {
            price: stop_loss,
            is_cleared: stop_loss_cleared,
        },
    );
    (!lines.is_empty()).then_some(GemListRow::Lines {
        title: GemListRowTitle::AutoClose,
        lines,
        info: None,
    })
}

pub fn chart_layout(candles: &[ChartCandleStick], position: Option<&PerpetualPosition>) -> GemPerpetualChartLayout {
    let candle_low = candles.iter().map(|candle| candle.low).reduce(f64::min).unwrap_or(0.0);
    let candle_high = candles.iter().map(|candle| candle.high).reduce(f64::max).unwrap_or(1.0);
    let buffer = (candle_high - candle_low) * CHART_LINE_VISIBILITY_BUFFER_FRACTION;
    let mut lines: Vec<(GemPerpetualChartLineKind, f64)> = position
        .map(chart_lines)
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, price)| *price >= candle_low - buffer && *price <= candle_high + buffer)
        .collect();
    lines.sort_by(|a, b| a.1.total_cmp(&b.1));
    let lowest = lines.first().map_or(candle_low, |(_, price)| price.min(candle_low));
    let highest = lines.last().map_or(candle_high, |(_, price)| price.max(candle_high));
    let span = (highest - lowest).max(highest.abs() * CHART_MINIMUM_SPAN_FRACTION + CHART_MINIMUM_SPAN);
    let padding = span * CHART_RANGE_PADDING_FRACTION;
    let price_low = if lowest > 0.0 { (lowest - padding).max(lowest * CHART_RANGE_FLOOR_FRACTION) } else { lowest - padding };
    let price_high = highest + padding;
    let overlap_threshold = (price_high - price_low) * CHART_LABEL_OVERLAP_FRACTION;
    let mut previous: Option<(f64, u32)> = None;
    let lines = lines
        .into_iter()
        .map(|(kind, price)| {
            let overlap_level = match previous {
                Some((last, level)) if price - last < overlap_threshold => level + 1,
                _ => 0,
            };
            previous = Some((price, overlap_level));
            GemPerpetualChartLine {
                kind,
                price: GemFormattedNumber::adaptive(price, None),
                overlap_level,
            }
        })
        .collect();
    GemPerpetualChartLayout {
        price_low,
        price_high,
        ticks: chart_ticks(candle_low, candle_high).into_iter().map(|tick| GemFormattedNumber::adaptive(tick, None)).collect(),
        x_tick_count: chart_x_tick_count(candles.len()),
        lines,
        current_price: candles.last().map(|candle| GemFormattedNumber::adaptive(candle.close, None)),
        tones: candles.iter().map(|candle| value_tone(candle.close - candle.open)).collect(),
    }
}

fn chart_x_tick_count(candle_count: usize) -> u32 {
    if candle_count < 2 { 0 } else { CHART_X_TICK_COUNT.min(candle_count) as u32 }
}

fn chart_ticks(candle_low: f64, candle_high: f64) -> Vec<f64> {
    if candle_high <= candle_low {
        return vec![candle_low];
    }
    let step = (candle_high - candle_low) / (CHART_TICK_COUNT - 1) as f64;
    (0..CHART_TICK_COUNT).map(|index| candle_low + step * index as f64).collect()
}

fn chart_lines(position: &PerpetualPosition) -> Vec<(GemPerpetualChartLineKind, f64)> {
    [
        (GemPerpetualChartLineKind::Entry, Some(position.entry_price)),
        (GemPerpetualChartLineKind::TakeProfit, position.take_profit.as_ref().map(|order| order.price)),
        (GemPerpetualChartLineKind::StopLoss, position.stop_loss.as_ref().map(|order| order.price)),
        (GemPerpetualChartLineKind::Liquidation, position.liquidation_price.filter(|price| *price > 0.0)),
    ]
    .into_iter()
    .filter_map(|(kind, price)| price.map(|price| (kind, price)))
    .collect()
}

pub fn funding_apr(funding: f64) -> f64 {
    funding * HOURS_PER_YEAR
}

pub fn slippage_percent(slippage: Option<f64>) -> f64 {
    slippage.unwrap_or(DEFAULT_SLIPPAGE_PERCENT)
}

impl GemPerpetualOrderAction {
    fn opens_position(&self) -> bool {
        match self {
            Self::Open | Self::Increase => true,
            Self::Reduce { .. } => false,
        }
    }
}

fn slippage_price(market_price: f64, direction: PerpetualDirection, opens: bool, slippage: f64) -> f64 {
    let fraction = slippage / 100.0;
    let multiplier = match (direction, opens) {
        (PerpetualDirection::Long, true) | (PerpetualDirection::Short, false) => 1.0 + fraction,
        (PerpetualDirection::Long, false) | (PerpetualDirection::Short, true) => 1.0 - fraction,
    };
    market_price * multiplier
}

fn order_amounts(usd_amount: f64, leverage: u8, price: f64) -> (f64, f64, f64) {
    let size = (usd_amount * f64::from(leverage)) / price;
    let fiat_value = price * size;
    let margin_amount = fiat_value / f64::from(leverage);
    (size, fiat_value, margin_amount)
}

pub fn includes_perpetual_collateral(mode: PerpetualAccountMode) -> bool {
    match mode {
        PerpetualAccountMode::Standard => true,
        PerpetualAccountMode::Unified => false,
    }
}

pub fn show_perpetuals(enabled: bool, wallet_type: WalletType, chains: &[Chain]) -> bool {
    enabled && supports_perpetuals(wallet_type, chains)
}

pub fn supports_perpetuals(wallet_type: WalletType, chains: &[Chain]) -> bool {
    wallet_type == WalletType::Multicoin && chains.iter().any(|chain| crate::services::stream::rules::is_hyperliquid_chain(*chain))
}

fn is_markets_stale(updated_at: Option<i64>, now: i64) -> bool {
    updated_at.is_none_or(|updated_at| now - updated_at >= MARKETS_REFRESH_INTERVAL_SECONDS)
}

impl GemMarketsRefreshTrigger {
    pub(super) fn should_sync_markets(self, updated_at: Option<i64>, now: i64) -> bool {
        match self {
            Self::UserRequested => true,
            Self::Scheduled => is_markets_stale(updated_at, now),
        }
    }
}

pub fn balance_update(balance: &PerpetualBalance) -> Result<GemBalanceUpdate, NumberFormatterError> {
    let asset = &*HYPERCORE_PERPETUAL_USDC;
    let value = |amount: f64| BigNumberFormatter::value_from_amount_biguint(&amount.to_string(), asset.decimals as u32);
    Ok(GemBalanceUpdate {
        asset_id: asset.id.clone(),
        update_type: GemBalanceUpdateType::Perpetual {
            available: value(balance.available)?,
            reserved: value(balance.reserved)?,
            withdrawable: value(balance.withdrawable)?,
        },
        is_active: true,
    })
}

pub fn provider(chain: Chain) -> Option<PerpetualProvider> {
    match chain {
        Chain::HyperCore | Chain::Hyperliquid => Some(PerpetualProvider::Hypercore),
        _ => None,
    }
}

pub fn prices_outdated(updated_at: Option<i64>, now: i64, interval_seconds: u32) -> bool {
    updated_at.is_none_or(|updated_at| now - updated_at >= i64::from(interval_seconds))
}

pub fn stale_position_ids(existing_ids: Vec<String>, positions: &[PerpetualPosition]) -> Vec<String> {
    stale(existing_ids, positions.iter().map(|position| position.id.clone()))
}

pub fn collateral_asset_id(chain: Chain) -> Option<AssetId> {
    wallet_default_assets(chain).into_iter().find(|asset| asset.asset_type == AssetType::PERPETUAL).map(|asset| asset.id)
}

pub fn balance_total(balance: Option<&PerpetualBalance>) -> GemFormattedNumber {
    GemFormattedNumber::usd(balance.map_or(0.0, |balance| balance.available + balance.reserved))
}

pub fn balance_header(balance: Option<PerpetualBalance>, wallet_type: WalletType) -> GemPerpetualBalanceHeader {
    let (available, withdrawable) = balance.as_ref().map_or((0.0, 0.0), |balance| (balance.available, balance.withdrawable));
    let perpetual = GemPerpetual::new(PerpetualProvider::Hypercore);
    GemPerpetualBalanceHeader {
        total: balance_total(balance.as_ref()),
        available: GemFormattedNumber::usd(available),
        deposit_asset: perpetual.deposit_asset(),
        withdraw_asset: HYPERCORE_PERPETUAL_USDC.clone(),
        actions: match wallet_type {
            WalletType::View => GemHeaderActions::WatchOnly,
            WalletType::Multicoin | WalletType::Single | WalletType::PrivateKey => GemHeaderActions::Buttons {
                buttons: vec![
                    GemHeaderButton {
                        kind: GemHeaderButtonKind::Withdraw,
                        is_enabled: withdrawable > 0.0,
                    },
                    GemHeaderButton {
                        kind: GemHeaderButtonKind::Deposit,
                        is_enabled: true,
                    },
                ],
            },
        },
    }
}

pub fn collateral_asset_ids() -> Vec<AssetId> {
    PerpetualProvider::iter().filter_map(|provider| collateral_asset_id(provider_chain(&provider))).collect()
}

fn provider_chain(provider: &PerpetualProvider) -> Chain {
    match provider {
        PerpetualProvider::Hypercore => Chain::HyperCore,
    }
}

pub fn collateral_price(chain: Chain) -> Option<AssetPrice> {
    collateral_asset_id(chain).map(|asset_id| AssetPrice::new(asset_id, 1.0, 0.0, Utc::now()))
}

pub fn order(provider: PerpetualProvider, input: GemPerpetualOrderInput) -> PerpetualType {
    let usd_amount = BigNumberFormatter::f64_value(&input.usdc_value, u32::try_from(input.usdc_decimals).unwrap_or_default());
    let slippage = slippage_percent(input.slippage);
    let (size, fiat_value, margin_amount) = order_amounts(usd_amount, input.leverage, input.price);
    let price = slippage_price(input.price, input.direction.clone(), input.action.opens_position(), slippage);
    let formatter = GemPerpetual::new(provider);

    let data = PerpetualConfirmData {
        direction: input.direction,
        margin_type: input.margin_type,
        base_asset: input.base_asset,
        asset_index: input.asset_index,
        price: formatter.format_price(price, input.asset.decimals),
        fiat_value,
        size: formatter.format_size(size, input.asset.decimals),
        slippage,
        leverage: input.leverage,
        pnl: None,
        entry_price: None,
        market_price: input.price,
        margin_amount,
        take_profit: input.take_profit,
        stop_loss: input.stop_loss,
    };

    match input.action {
        GemPerpetualOrderAction::Open => PerpetualType::Open { data },
        GemPerpetualOrderAction::Increase => PerpetualType::Increase { data },
        GemPerpetualOrderAction::Reduce { position_direction } => PerpetualType::Reduce {
            data: PerpetualReduceData { data, position_direction },
        },
    }
}

pub(super) fn asset_index(perpetual: &Perpetual) -> Result<i32, GemServiceError> {
    perpetual.identifier.parse().map_err(|_| GemServiceError::InvalidInput {
        msg: format!("perpetual {} has no asset index", perpetual.identifier),
    })
}

fn position_for(perpetual: &Perpetual, position: Option<PerpetualPosition>) -> Result<PerpetualPosition, GemServiceError> {
    position.ok_or_else(|| GemServiceError::InvalidInput {
        msg: format!("no position on {}", perpetual.identifier),
    })
}

pub(crate) fn margin_amount_value(position: &PerpetualPosition) -> BigUint {
    BigUint::from((position.margin_amount * 10f64.powi(HYPERCORE_PERPETUAL_USDC.decimals)).max(0.0) as u64)
}

pub fn position_action(perpetual: &Perpetual, asset: &Asset, position: Option<PerpetualPosition>, kind: GemPerpetualPositionKind) -> Result<GemPerpetualPositionAction, GemServiceError> {
    let asset_index = asset_index(perpetual)?;
    let data = |direction: PerpetualDirection, leverage: u8, margin_type: PerpetualMarginType| GemPerpetualTransferData {
        provider: perpetual.provider.clone(),
        direction,
        asset: asset.clone(),
        base_asset: HYPERCORE_PERPETUAL_USDC.clone(),
        asset_index,
        price: perpetual.price,
        leverage,
        margin_type,
    };
    Ok(match kind {
        GemPerpetualPositionKind::Open { direction } => {
            let margin_type = if perpetual.is_isolated_only { PerpetualMarginType::Isolated } else { PerpetualMarginType::Cross };
            GemPerpetualPositionAction::Open {
                data: data(direction, perpetual.max_leverage, margin_type),
            }
        }
        GemPerpetualPositionKind::Increase => {
            let position = position_for(perpetual, position)?;
            GemPerpetualPositionAction::Increase {
                data: data(position.direction.clone(), position.leverage, position.margin_type.clone()),
            }
        }
        GemPerpetualPositionKind::Reduce => {
            let position = position_for(perpetual, position)?;
            GemPerpetualPositionAction::Reduce {
                data: data(position.direction.clone(), position.leverage, position.margin_type.clone()),
                position,
            }
        }
    })
}

pub fn close_transfer(perpetual: &Perpetual, asset: &Asset, position: Option<PerpetualPosition>) -> Result<GemTransferData, GemServiceError> {
    let position = position_for(perpetual, position)?;
    Ok(position_close_transfer(perpetual.provider.clone(), asset_index(perpetual)?, perpetual.price, asset.clone(), position))
}

fn position_close_transfer(provider: PerpetualProvider, asset_index: i32, market_price: f64, asset: Asset, position: PerpetualPosition) -> GemTransferData {
    let data = close_order(
        provider.clone(),
        GemPerpetualCloseInput {
            asset_index,
            direction: position.direction,
            margin_type: position.margin_type,
            base_asset: HYPERCORE_PERPETUAL_USDC.clone(),
            asset: asset.clone(),
            market_price,
            size: position.size,
            leverage: position.leverage,
            pnl: position.pnl,
            entry_price: position.entry_price,
            margin_amount: position.margin_amount,
            slippage: None,
        },
    );
    GemPerpetual::new(provider).transfer_data(asset, PerpetualType::Close { data }, GemBigInt::ZERO, false)
}

pub fn order_transfer(action: GemPerpetualPositionAction, usdc_value: GemBigInt, use_max_amount: bool, leverage: u8, take_profit: Option<f64>, stop_loss: Option<f64>) -> GemTransferData {
    let data = action.data().clone();
    let perpetual = GemPerpetual::new(data.provider.clone());
    let order_action = match &action {
        GemPerpetualPositionAction::Open { .. } => GemPerpetualOrderAction::Open,
        GemPerpetualPositionAction::Increase { .. } => GemPerpetualOrderAction::Increase,
        GemPerpetualPositionAction::Reduce { position, .. } if usdc_value > GemBigInt::ZERO && usdc_value == GemBigInt::from(margin_amount_value(position)) => {
            return position_close_transfer(data.provider, data.asset_index, data.price, data.asset, position.clone());
        }
        GemPerpetualPositionAction::Reduce { .. } => GemPerpetualOrderAction::Reduce { position_direction: data.direction.clone() },
    };
    let trigger = |price: Option<f64>| price.map(|price| perpetual.format_price(price, data.asset.decimals));
    let perpetual_type = order(
        data.provider.clone(),
        GemPerpetualOrderInput {
            action: order_action,
            direction: data.direction,
            margin_type: data.margin_type,
            base_asset: data.base_asset.clone(),
            asset: data.asset.clone(),
            asset_index: data.asset_index,
            price: data.price,
            usdc_value: usdc_value.clone(),
            usdc_decimals: data.base_asset.decimals,
            leverage,
            slippage: None,
            take_profit: trigger(take_profit),
            stop_loss: trigger(stop_loss),
        },
    );
    perpetual.transfer_data(data.asset, perpetual_type, usdc_value, use_max_amount)
}

pub fn close_order(provider: PerpetualProvider, input: GemPerpetualCloseInput) -> PerpetualConfirmData {
    let slippage = slippage_percent(input.slippage);
    let price = slippage_price(input.market_price, input.direction.clone(), false, slippage);
    let size = input.size.abs();
    let formatter = GemPerpetual::new(provider);

    PerpetualConfirmData {
        direction: input.direction,
        margin_type: input.margin_type,
        base_asset: input.base_asset,
        asset_index: input.asset_index,
        price: formatter.format_price(price, input.asset.decimals),
        fiat_value: size * price,
        size: formatter.format_size(size, input.asset.decimals),
        slippage,
        leverage: input.leverage,
        pnl: Some(input.pnl),
        entry_price: Some(input.entry_price),
        market_price: input.market_price,
        margin_amount: input.margin_amount,
        take_profit: None,
        stop_loss: None,
    }
}

pub fn symbol(perpetual: &Perpetual) -> String {
    perpetual.name.clone()
}

pub fn merged_candles(candles: Vec<ChartCandleStick>, update: ChartCandleUpdate, perpetual: &Perpetual, period: &ChartPeriod) -> Option<Vec<ChartCandleStick>> {
    (update.coin == symbol(perpetual) && update.interval == candle_interval(period)).then(|| merge_candle(candles, update.candle))
}

fn merge_candle(candles: Vec<ChartCandleStick>, candle: ChartCandleStick) -> Vec<ChartCandleStick> {
    let Some(last_date) = candles.last().map(|last| last.date) else {
        return candles;
    };
    let mut merged = candles;
    match candle.date.cmp(&last_date) {
        Ordering::Less => return merged,
        Ordering::Equal => {
            merged.pop();
        }
        Ordering::Greater => {
            merged.remove(0);
        }
    }
    merged.push(candle);
    merged
}

pub const MARKETS_LIMIT: u32 = 100;

pub fn market_query(search: String) -> GemPerpetualMarketQuery {
    GemPerpetualMarketQuery {
        requires_volume: search.is_empty(),
        search,
        limit: MARKETS_LIMIT,
    }
}

pub fn market_sections(counts: &GemPerpetualMarketCounts, is_searching: bool, is_query_empty: bool) -> Vec<GemPerpetualMarketSection> {
    let shows_positions = counts.positions > 0;
    let shows_pinned = counts.pinned > 0;
    let shows_markets = counts.markets > 0;
    [
        (is_searching && is_query_empty && counts.recents > 0, GemPerpetualMarketSection::Recents),
        (shows_positions, GemPerpetualMarketSection::Positions),
        (shows_pinned, GemPerpetualMarketSection::Pinned),
        (shows_markets, GemPerpetualMarketSection::Markets),
        (is_searching && !shows_positions && !shows_pinned && !shows_markets, GemPerpetualMarketSection::Empty),
    ]
    .into_iter()
    .filter_map(|(shows, section)| shows.then_some(section))
    .collect()
}

pub fn candle_tooltip(candle: &ChartCandleStick) -> GemCandleTooltip {
    GemCandleTooltip {
        prices: vec![
            tooltip_cell(GemCandleTooltipRow::Open, GemFormattedNumber::adaptive(candle.open, None)),
            tooltip_cell(GemCandleTooltipRow::High, GemFormattedNumber::adaptive(candle.high, None)),
            tooltip_cell(GemCandleTooltipRow::Low, GemFormattedNumber::adaptive(candle.low, None)),
            tooltip_cell(GemCandleTooltipRow::Close, GemFormattedNumber::adaptive(candle.close, None)),
        ],
        summary: vec![
            tooltip_cell(
                GemCandleTooltipRow::Change,
                GemFormattedNumber::percentage(PriceChangeCalculator::percentage(candle.open, candle.close), GemPercentageStyle::Signed),
            ),
            tooltip_cell(GemCandleTooltipRow::Volume, GemFormattedNumber::usd_abbreviated(candle.volume * candle.close)),
        ],
    }
}

fn tooltip_cell(row: GemCandleTooltipRow, value: GemFormattedNumber) -> GemCandleTooltipCell {
    GemCandleTooltipCell { row, value }
}

pub fn market_row(perpetual: &Perpetual, asset: &Asset) -> GemPerpetualMarketRow {
    GemPerpetualMarketRow {
        icon: crate::services::assets::icon::asset_icon(&asset.id),
        asset_id: perpetual.asset_id.clone(),
        title: match perpetual.name.is_empty() {
            true => asset.symbol.clone(),
            false => perpetual.name.clone(),
        },
        price: crate::services::assets::rules::price_row(Some(perpetual.price), Some(perpetual.price_percent_change_24h), Currency::USD, GemCurrencyStyle::Short),
        volume_24h: GemFormattedNumber::usd_abbreviated(perpetual.volume_24h),
        open_interest: GemFormattedNumber::usd_abbreviated(perpetual.open_interest),
        funding_apr: GemFormattedNumber::percentage(funding_apr(perpetual.funding), GemPercentageStyle::Signed),
    }
}

pub fn open_row(direction: PerpetualDirection, leverage: u8, size: f64) -> GemPerpetualOpenRow {
    GemPerpetualOpenRow {
        position: position_text(&direction, leverage),
        direction_tone: direction_tone(&direction),
        size: (size > 0.0).then(|| GemFormattedNumber::currency(size, Currency::USD, GemCurrencyStyle::Currency)),
    }
}

pub fn position_row(perpetual: &Perpetual, asset: &Asset, position: &PerpetualPosition) -> GemPerpetualPositionRow {
    let (pnl, pnl_tone) = pnl_text(position.pnl, position.margin_amount);
    GemPerpetualPositionRow {
        icon: crate::services::assets::icon::asset_icon(&asset.id),
        id: position.id.clone(),
        asset_id: perpetual.asset_id.clone(),
        title: match asset.symbol.is_empty() {
            true => perpetual.name.clone(),
            false => asset.symbol.clone(),
        },
        position: position_text(&position.direction, position.leverage),
        direction_tone: direction_tone(&position.direction),
        margin: GemFormattedNumber::currency(position.margin_amount, Currency::USD, GemCurrencyStyle::Fiat),
        direction: position.direction.clone(),
        pnl,
        pnl_tone,
    }
}

pub fn details(perpetual: &Perpetual, asset: &Asset, positions: Vec<PerpetualPosition>) -> GemPerpetualDetails {
    let position = positions.into_iter().next();
    let market = market_row(perpetual, asset);
    GemPerpetualDetails {
        title: market.title.clone(),
        sections: [
            position.as_ref().map(|position| GemPerpetualSection::Position { rows: position_details(position) }),
            Some(GemPerpetualSection::Info {
                buttons: perpetual_buttons(position.is_some()),
                rows: info_rows(market),
            }),
        ]
        .into_iter()
        .flatten()
        .collect(),
        modify_buttons: button_rows(&[GemPerpetualButton::Increase, GemPerpetualButton::Reduce]),
        position_row: position.as_ref().map(|position| position_row(perpetual, asset, position)),
        position,
    }
}

fn position_details(position: &PerpetualPosition) -> Vec<GemPerpetualPositionDetail> {
    let amount = |title: GemListRowTitle, amount: GemFormattedNumber, info: Option<GemInfoTopic>| GemListRow::Amount { title, amount, info };
    let label = |title: GemListRowTitle, text: GemLocalizedText, tone: GemValueTone, info: Option<GemInfoTopic>| GemListRow::Label { title, text, tone, info, progress: false };
    let pnl = GemLocalizedText::Pnl {
        amount: GemFormattedNumber::signed_usd(position.pnl),
        percent: GemFormattedNumber::percentage(PriceChangeCalculator::pnl_percentage(position.pnl, position.margin_amount), GemPercentageStyle::Signed),
    };
    let margin = GemLocalizedText::Margin {
        amount: GemFormattedNumber::usd(position.margin_amount),
        margin_type: position.margin_type.clone(),
    };
    let liquidation_price = position.liquidation_price.filter(|price| *price > 0.0).map(|price| {
        amount(
            GemListRowTitle::LiquidationPrice,
            GemFormattedNumber {
                tone: GemValueTone::Neutral,
                ..GemFormattedNumber::usd(price)
            },
            Some(GemInfoTopic::LiquidationPrice),
        )
    });
    let funding = match position.funding {
        Some(funding) => amount(GemListRowTitle::FundingPayments, GemFormattedNumber::signed_usd(funding as f64), Some(GemInfoTopic::FundingPayments)),
        None => label(
            GemListRowTitle::FundingPayments,
            GemLocalizedText::Text { text: EMPTY_VALUE.to_string() },
            GemValueTone::Neutral,
            Some(GemInfoTopic::FundingPayments),
        ),
    };
    [
        (GemPerpetualPositionDetailRow::Pnl, Some(label(GemListRowTitle::Pnl, pnl, GemValueTone::of(position.pnl), None))),
        (
            GemPerpetualPositionDetailRow::Autoclose,
            Some(GemListRow::Lines {
                title: GemListRowTitle::AutoClose,
                lines: autoclose_lines(position),
                info: Some(GemInfoTopic::AutoClose),
            }),
        ),
        (GemPerpetualPositionDetailRow::Size, Some(amount(GemListRowTitle::Size, GemFormattedNumber::usd(position.size_value), None))),
        (GemPerpetualPositionDetailRow::EntryPrice, Some(amount(GemListRowTitle::EntryPrice, GemFormattedNumber::usd(position.entry_price), None))),
        (GemPerpetualPositionDetailRow::LiquidationPrice, liquidation_price),
        (GemPerpetualPositionDetailRow::Margin, Some(label(GemListRowTitle::Margin, margin, GemValueTone::Plain, None))),
        (GemPerpetualPositionDetailRow::FundingPayments, Some(funding)),
    ]
    .into_iter()
    .filter_map(|(kind, row)| row.map(|row| GemPerpetualPositionDetail { kind, row }))
    .collect()
}

fn autoclose_lines(position: &PerpetualPosition) -> Vec<GemLocalizedText> {
    let line = |order: GemTriggerOrder, trigger: &Option<primitives::PerpetualTriggerOrder>| {
        trigger.as_ref().map(|trigger| GemLocalizedText::TriggerOrder {
            order,
            price: Some(GemFormattedNumber::usd(trigger.price)),
        })
    };
    let lines: Vec<GemLocalizedText> = [line(GemTriggerOrder::TakeProfit, &position.take_profit), line(GemTriggerOrder::StopLoss, &position.stop_loss)].into_iter().flatten().collect();
    match lines.is_empty() {
        true => vec![GemLocalizedText::Text { text: EMPTY_VALUE.to_string() }],
        false => lines,
    }
}

fn info_rows(row: GemPerpetualMarketRow) -> Vec<GemListRow> {
    vec![
        GemListRow::Amount {
            title: GemListRowTitle::DailyVolume,
            amount: row.volume_24h,
            info: None,
        },
        GemListRow::Amount {
            title: GemListRowTitle::OpenInterest,
            amount: row.open_interest,
            info: Some(GemInfoTopic::OpenInterest),
        },
        GemListRow::Amount {
            title: GemListRowTitle::FundingApr,
            amount: row.funding_apr,
            info: Some(GemInfoTopic::FundingApr),
        },
    ]
}

fn perpetual_buttons(has_position: bool) -> Vec<GemPerpetualButtonRow> {
    match has_position {
        true => button_rows(&[GemPerpetualButton::Modify, GemPerpetualButton::Close]),
        false => button_rows(&[GemPerpetualButton::Long, GemPerpetualButton::Short]),
    }
}

fn button_rows(buttons: &[GemPerpetualButton]) -> Vec<GemPerpetualButtonRow> {
    buttons.iter().map(|button| GemPerpetualButtonRow { button: *button, tone: button_tone(button) }).collect()
}

fn button_tone(button: &GemPerpetualButton) -> GemValueTone {
    match button {
        GemPerpetualButton::Long => GemValueTone::Positive,
        GemPerpetualButton::Short | GemPerpetualButton::Close | GemPerpetualButton::Reduce => GemValueTone::Negative,
        GemPerpetualButton::Modify | GemPerpetualButton::Increase => GemValueTone::Neutral,
    }
}

pub fn changed_perpetuals(data: Vec<PerpetualData>, stored: &[Perpetual]) -> Vec<PerpetualData> {
    data.into_iter().filter(|data| !stored.contains(&data.perpetual)).collect()
}

pub fn market_changed(market: &PerpetualMarketData, stored: &[Perpetual]) -> bool {
    !stored.iter().any(|perpetual| {
        perpetual.name == market.coin
            && perpetual.price == market.price
            && perpetual.price_percent_change_24h == market.price_percent_change_24h
            && perpetual.open_interest == market.open_interest
            && perpetual.volume_24h == market.volume_24h
            && perpetual.funding == market.funding
    })
}

pub fn changed_perpetual_prices(prices: HashMap<String, f64>, stored: &[Perpetual]) -> HashMap<String, f64> {
    prices.into_iter().filter(|(name, price)| !stored.iter().any(|perpetual| perpetual.name == *name && perpetual.price == *price)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::amount::{GemAmountPerpetualPosition, GemAmountType, rules::perpetual_amount_type};
    use crate::services::perpetual::model::GemPerpetualMarketSession;
    use num_bigint::BigInt;
    use num_bigint::BigUint;
    use primitives::PerpetualTriggerOrder;
    use primitives::TransactionInputType;

    #[test]
    fn test_the_balance_header_names_the_asset_each_button_moves() {
        let header = balance_header(None, WalletType::Multicoin);

        assert_eq!(header.deposit_asset, GemPerpetual::new(PerpetualProvider::Hypercore).deposit_asset());
        assert_eq!(header.withdraw_asset, *HYPERCORE_PERPETUAL_USDC, "a withdrawal leaves the perpetual account, not the chain");
    }

    #[test]
    fn test_each_button_carries_the_tone_it_is_drawn_in() {
        let tones: Vec<(GemPerpetualButton, GemValueTone)> = button_rows(&[
            GemPerpetualButton::Long,
            GemPerpetualButton::Short,
            GemPerpetualButton::Modify,
            GemPerpetualButton::Close,
            GemPerpetualButton::Increase,
            GemPerpetualButton::Reduce,
        ])
        .into_iter()
        .map(|row| (row.button, row.tone))
        .collect();

        assert_eq!(
            tones,
            vec![
                (GemPerpetualButton::Long, GemValueTone::Positive),
                (GemPerpetualButton::Short, GemValueTone::Negative),
                (GemPerpetualButton::Modify, GemValueTone::Neutral),
                (GemPerpetualButton::Close, GemValueTone::Negative),
                (GemPerpetualButton::Increase, GemValueTone::Neutral),
                (GemPerpetualButton::Reduce, GemValueTone::Negative),
            ]
        );
    }

    #[test]
    fn test_the_perpetual_screen_names_its_sections_rows_and_buttons_from_the_position() {
        let perpetual = Perpetual::mock();
        let asset = Asset::mock();
        let position = PerpetualPosition::mock();
        let modify_buttons = button_rows(&[GemPerpetualButton::Increase, GemPerpetualButton::Reduce]);

        assert_eq!(
            details(&perpetual, &asset, vec![]),
            GemPerpetualDetails {
                title: market_row(&perpetual, &asset).title,
                sections: vec![GemPerpetualSection::Info {
                    buttons: button_rows(&[GemPerpetualButton::Long, GemPerpetualButton::Short]),
                    rows: info_rows(market_row(&perpetual, &asset)),
                }],
                modify_buttons: modify_buttons.clone(),
                position: None,
                position_row: None,
            }
        );
        assert_eq!(
            details(&perpetual, &asset, vec![position.clone()]),
            GemPerpetualDetails {
                title: market_row(&perpetual, &asset).title,
                sections: vec![
                    GemPerpetualSection::Position { rows: position_details(&position) },
                    GemPerpetualSection::Info {
                        buttons: button_rows(&[GemPerpetualButton::Modify, GemPerpetualButton::Close]),
                        rows: info_rows(market_row(&perpetual, &asset)),
                    },
                ],
                modify_buttons,
                position_row: Some(position_row(&perpetual, &asset, &position)),
                position: Some(position),
            }
        );

        let detail_kinds = |position: &PerpetualPosition| position_details(position).into_iter().map(|detail| detail.kind).collect::<Vec<_>>();

        let without_liquidation = PerpetualPosition {
            liquidation_price: Some(0.0),
            ..PerpetualPosition::mock()
        };
        assert!(
            !detail_kinds(&without_liquidation).contains(&GemPerpetualPositionDetailRow::LiquidationPrice),
            "a zero liquidation price is no liquidation price"
        );
        let unliquidatable = PerpetualPosition {
            liquidation_price: None,
            ..PerpetualPosition::mock()
        };
        assert!(!detail_kinds(&unliquidatable).contains(&GemPerpetualPositionDetailRow::LiquidationPrice));
        let liquidatable = PerpetualPosition {
            liquidation_price: Some(1.0),
            ..without_liquidation
        };
        assert_eq!(
            detail_kinds(&liquidatable),
            vec![
                GemPerpetualPositionDetailRow::Pnl,
                GemPerpetualPositionDetailRow::Autoclose,
                GemPerpetualPositionDetailRow::Size,
                GemPerpetualPositionDetailRow::EntryPrice,
                GemPerpetualPositionDetailRow::LiquidationPrice,
                GemPerpetualPositionDetailRow::Margin,
                GemPerpetualPositionDetailRow::FundingPayments,
            ]
        );
    }

    #[test]
    fn test_market_sections_hide_recents_mid_search_and_answer_empty_only_while_searching() {
        let counts = GemPerpetualMarketCounts {
            positions: 0,
            pinned: 0,
            markets: 0,
            recents: 2,
        };

        assert_eq!(market_sections(&counts, true, true), vec![GemPerpetualMarketSection::Recents, GemPerpetualMarketSection::Empty]);
        assert_eq!(market_sections(&counts, true, false), vec![GemPerpetualMarketSection::Empty]);
        assert_eq!(market_sections(&counts, false, true), vec![]);

        let listed = GemPerpetualMarketCounts {
            positions: 1,
            pinned: 2,
            markets: 3,
            recents: 0,
        };
        assert_eq!(
            market_sections(&listed, true, true),
            vec![GemPerpetualMarketSection::Positions, GemPerpetualMarketSection::Pinned, GemPerpetualMarketSection::Markets]
        );
    }

    #[test]
    fn test_market_session_trims_the_query_and_keeps_recents_until_one_is_typed() {
        let counts = GemPerpetualMarketCounts {
            positions: 0,
            pinned: 0,
            markets: 0,
            recents: 2,
        };
        let searching = GemPerpetualMarketSession::default().on_searching_changed(true);

        assert_eq!(searching.sections(counts), vec![GemPerpetualMarketSection::Recents, GemPerpetualMarketSection::Empty]);
        assert_eq!(searching.on_query_changed("  ".to_string()).sections(counts), searching.sections(counts));
        assert_eq!(searching.on_query_changed(" btc ".to_string()).search_query(), "btc");
        assert_eq!(searching.on_query_changed(" btc ".to_string()).sections(counts), vec![GemPerpetualMarketSection::Empty]);
        assert_eq!(GemPerpetualMarketSession::default().sections(counts), vec![]);
    }

    #[test]
    fn test_autoclose_row_reads_new_prices_and_cleared_orders() {
        let row = |lines: Vec<GemLocalizedText>| GemListRow::Lines {
            title: GemListRowTitle::AutoClose,
            lines,
            info: None,
        };
        let order = |order: GemTriggerOrder, price: Option<f64>| GemLocalizedText::TriggerOrder {
            order,
            price: price.map(GemFormattedNumber::usd),
        };

        assert_eq!(
            autoclose_row(&PerpetualModifyConfirmData::mock(vec![PerpetualModifyPositionType::mock_tpsl(Some("65000"), Some("55000"))], None, None,)),
            Some(row(vec![order(GemTriggerOrder::TakeProfit, Some(65000.0)), order(GemTriggerOrder::StopLoss, Some(55000.0))]))
        );
        assert_eq!(
            autoclose_row(&PerpetualModifyConfirmData::mock(vec![PerpetualModifyPositionType::mock_cancel(vec![111, 222])], Some(111), Some(222),)),
            Some(row(vec![order(GemTriggerOrder::TakeProfit, None), order(GemTriggerOrder::StopLoss, None)])),
            "a cleared order reads as an empty price"
        );
        assert_eq!(
            autoclose_row(&PerpetualModifyConfirmData::mock(
                vec![PerpetualModifyPositionType::mock_tpsl(Some("70000"), None), PerpetualModifyPositionType::mock_cancel(vec![111]),],
                Some(111),
                None,
            )),
            Some(row(vec![order(GemTriggerOrder::TakeProfit, Some(70000.0))])),
            "a replaced order is not a cleared one"
        );
        assert_eq!(
            autoclose_row(&PerpetualModifyConfirmData::mock(vec![PerpetualModifyPositionType::mock_tpsl(None, Some("50000"))], None, None)),
            Some(row(vec![order(GemTriggerOrder::StopLoss, Some(50000.0))]))
        );

        assert!(autoclose_row(&PerpetualModifyConfirmData::mock(vec![], None, None)).is_none());
        assert!(
            autoclose_row(&PerpetualModifyConfirmData::mock(vec![PerpetualModifyPositionType::mock_cancel(vec![999])], Some(111), None)).is_none(),
            "cancelling an unrelated order leaves nothing to show"
        );
    }

    #[test]
    fn test_position_details_carry_finished_values() {
        let position = PerpetualPosition {
            pnl: 5.0,
            funding: Some(-1.5),
            take_profit: Some(primitives::PerpetualTriggerOrder {
                price: 120.0,
                order_type: primitives::PerpetualOrderType::Market,
                order_id: "1".to_string(),
            }),
            ..PerpetualPosition::mock()
        };
        let row = |kind: GemPerpetualPositionDetailRow| position_details(&position).into_iter().find(|detail| detail.kind == kind).unwrap().row;

        assert_eq!(
            row(GemPerpetualPositionDetailRow::Pnl),
            GemListRow::Label {
                title: GemListRowTitle::Pnl,
                text: GemLocalizedText::Pnl {
                    amount: GemFormattedNumber::signed_usd(5.0),
                    percent: GemFormattedNumber::percentage(25.0, GemPercentageStyle::Signed),
                },
                tone: GemValueTone::Positive,
                info: None,
                progress: false,
            }
        );
        assert_eq!(
            row(GemPerpetualPositionDetailRow::Autoclose),
            GemListRow::Lines {
                title: GemListRowTitle::AutoClose,
                lines: vec![GemLocalizedText::TriggerOrder {
                    order: GemTriggerOrder::TakeProfit,
                    price: Some(GemFormattedNumber::usd(120.0)),
                }],
                info: Some(GemInfoTopic::AutoClose),
            }
        );
        assert_eq!(
            row(GemPerpetualPositionDetailRow::FundingPayments),
            GemListRow::Amount {
                title: GemListRowTitle::FundingPayments,
                amount: GemFormattedNumber::signed_usd(-1.5),
                info: Some(GemInfoTopic::FundingPayments),
            }
        );
        assert!(matches!(
            row(GemPerpetualPositionDetailRow::Margin),
            GemListRow::Label {
                text: GemLocalizedText::Margin { margin_type: PerpetualMarginType::Cross, .. },
                ..
            }
        ));
        assert_eq!(
            position_details(&PerpetualPosition::mock())
                .into_iter()
                .find(|detail| detail.kind == GemPerpetualPositionDetailRow::Autoclose)
                .map(|detail| detail.row),
            Some(GemListRow::Lines {
                title: GemListRowTitle::AutoClose,
                lines: vec![GemLocalizedText::Text { text: "-".to_string() }],
                info: Some(GemInfoTopic::AutoClose),
            }),
            "no trigger orders read as an empty value"
        );
    }

    #[test]
    fn test_perpetual_collateral_counts_only_in_standard_mode() {
        assert!(includes_perpetual_collateral(PerpetualAccountMode::Standard));
        assert!(!includes_perpetual_collateral(PerpetualAccountMode::Unified));
    }

    #[test]
    fn test_chart_layout_pads_the_candle_range_and_draws_four_ticks() {
        let layout = chart_layout(&[ChartCandleStick::mock_range(9.0, 12.0), ChartCandleStick::mock_range(10.0, 13.0)], None);

        assert!(layout.price_low < 9.0 && layout.price_low >= 9.0 * CHART_RANGE_FLOOR_FRACTION);
        assert!(layout.price_high > 13.0);
        assert_eq!(layout.ticks.len(), 4);
        assert_eq!(layout.ticks[0].value, 9.0);
        assert_eq!(layout.ticks[3].value, 13.0);
        assert!(layout.lines.is_empty());
    }

    #[test]
    fn test_chart_layout_colours_each_candle_by_its_move() {
        let rising = ChartCandleStick::mock_range(9.0, 12.0);
        let falling = ChartCandleStick { open: 12.0, close: 9.0, ..rising.clone() };
        let flat = ChartCandleStick { open: 10.0, close: 10.0, ..rising.clone() };

        let layout = chart_layout(&[rising, falling, flat], None);

        assert_eq!(layout.tones, vec![GemValueTone::Positive, GemValueTone::Negative, GemValueTone::Neutral]);
    }

    #[test]
    fn test_chart_x_tick_count_never_exceeds_the_candles_it_can_mark() {
        assert_eq!(chart_x_tick_count(0), 0, "an empty series draws no gridlines");
        assert_eq!(chart_x_tick_count(1), 0, "a single candle spans nothing to divide");
        assert_eq!(chart_x_tick_count(4), 4);
        assert_eq!(chart_x_tick_count(40), CHART_X_TICK_COUNT as u32);
    }

    #[test]
    fn test_chart_layout_keeps_the_lines_near_the_candles_sorted_and_inside_the_range() {
        let mut open = PerpetualPosition::mock();
        open.entry_price = 14.0;
        open.stop_loss = Some(PerpetualTriggerOrder::mock(8.0));
        open.take_profit = Some(PerpetualTriggerOrder::mock(100.0));
        open.liquidation_price = Some(1.0);

        let layout = chart_layout(&[ChartCandleStick::mock_range(9.0, 13.0)], Some(&open));
        let lines: Vec<(GemPerpetualChartLineKind, f64)> = layout.lines.iter().map(|line| (line.kind, line.price.value)).collect();

        assert_eq!(lines, vec![(GemPerpetualChartLineKind::StopLoss, 8.0), (GemPerpetualChartLineKind::Entry, 14.0)]);
        assert!(layout.price_low <= 8.0 && layout.price_high >= 14.0);
    }

    #[test]
    fn test_chart_layout_levels_the_labels_that_would_overlap() {
        let mut open = PerpetualPosition::mock();
        open.entry_price = 121.0;
        open.take_profit = Some(PerpetualTriggerOrder::mock(180.0));
        open.liquidation_price = Some(120.0);

        let layout = chart_layout(&[ChartCandleStick::mock_range(100.0, 200.0)], Some(&open));
        let levels: Vec<(GemPerpetualChartLineKind, u32)> = layout.lines.iter().map(|line| (line.kind, line.overlap_level)).collect();

        assert_eq!(levels, vec![(GemPerpetualChartLineKind::Liquidation, 0), (GemPerpetualChartLineKind::Entry, 1), (GemPerpetualChartLineKind::TakeProfit, 0)]);
    }

    #[test]
    fn test_chart_layout_keeps_a_measurable_range_for_flat_and_negative_series() {
        let flat = chart_layout(&[ChartCandleStick::mock_range(100.0, 100.0)], None);
        assert!(flat.price_low < flat.price_high);
        assert_eq!(flat.ticks.iter().map(|tick| tick.value).collect::<Vec<_>>(), vec![100.0]);

        let negative = chart_layout(&[ChartCandleStick::mock_range(-10.0, -5.0)], None);
        assert!(negative.price_low < -10.0);
    }

    #[test]
    fn test_chart_lines_show_the_prices_a_position_has() {
        let mut open = PerpetualPosition::mock();
        open.entry_price = 100.0;
        open.take_profit = Some(PerpetualTriggerOrder::mock(120.0));
        open.stop_loss = Some(PerpetualTriggerOrder::mock(90.0));
        open.liquidation_price = Some(80.0);
        let kinds: Vec<(GemPerpetualChartLineKind, f64)> = chart_lines(&open);
        assert_eq!(
            kinds,
            vec![
                (GemPerpetualChartLineKind::Entry, 100.0),
                (GemPerpetualChartLineKind::TakeProfit, 120.0),
                (GemPerpetualChartLineKind::StopLoss, 90.0),
                (GemPerpetualChartLineKind::Liquidation, 80.0),
            ]
        );

        let mut bare = PerpetualPosition::mock();
        bare.liquidation_price = Some(0.0);
        assert_eq!(chart_lines(&bare).len(), 1, "a zero liquidation price and missing orders draw only the entry line");
    }

    #[test]
    fn test_markets_stale_after_an_hour_or_when_never_synced() {
        assert!(is_markets_stale(None, 10_000));
        assert!(!is_markets_stale(Some(10_000 - 3_599), 10_000));
        assert!(is_markets_stale(Some(10_000 - 3_600), 10_000));
    }

    #[test]
    fn test_a_user_requested_refresh_syncs_markets_that_a_scheduled_one_would_skip() {
        let just_synced = Some(10_000 - 1);

        assert!(!GemMarketsRefreshTrigger::Scheduled.should_sync_markets(just_synced, 10_000));
        assert!(GemMarketsRefreshTrigger::UserRequested.should_sync_markets(just_synced, 10_000));
    }

    #[test]
    fn test_a_candle_tooltip_quotes_volume_in_the_close_price() {
        let candle = ChartCandleStick {
            date: Utc::now(),
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 110.0,
            volume: 2.0,
        };
        let tooltip = candle_tooltip(&candle);

        assert_eq!(
            tooltip.prices.iter().map(|cell| cell.row).collect::<Vec<_>>(),
            vec![GemCandleTooltipRow::Open, GemCandleTooltipRow::High, GemCandleTooltipRow::Low, GemCandleTooltipRow::Close]
        );
        assert_eq!(tooltip.summary.iter().map(|cell| cell.row).collect::<Vec<_>>(), vec![GemCandleTooltipRow::Change, GemCandleTooltipRow::Volume]);
        assert_eq!(tooltip.prices[1].value, GemFormattedNumber::adaptive(120.0, None));
        assert_eq!(tooltip.summary[0].value, GemFormattedNumber::percentage(10.0, GemPercentageStyle::Signed));
        assert_eq!(tooltip.summary[1].value, GemFormattedNumber::usd_abbreviated(220.0));

        let flat = candle_tooltip(&ChartCandleStick { open: 0.0, ..candle });

        assert_eq!(flat.summary[0].value, GemFormattedNumber::percentage(0.0, GemPercentageStyle::Signed), "an open of nothing has no percentage to quote");
    }

    #[test]
    fn test_a_market_row_hides_a_price_it_does_not_have() {
        let priced = Perpetual::mock();
        let unpriced = Perpetual { price: 0.0, ..priced.clone() };

        let asset = Asset::from_chain(Chain::HyperCore);

        assert_eq!(market_row(&priced, &asset).title, "BTC");
        assert_eq!(
            market_row(&priced, &asset).price,
            crate::services::assets::rules::price_row(Some(priced.price), Some(priced.price_percent_change_24h), Currency::USD, GemCurrencyStyle::Short),
            "the row carries its price the way every other row does"
        );
        assert_eq!(market_row(&unpriced, &asset).price.price, None);
    }

    #[test]
    fn test_rows_carry_the_market_asset_and_the_position_id() {
        let perpetual = Perpetual::mock();
        let asset = Asset::from_chain(Chain::HyperCore);
        let position = PerpetualPosition::mock();

        assert_eq!(market_row(&perpetual, &asset).asset_id, perpetual.asset_id);
        let row = position_row(&perpetual, &asset, &position);
        assert_eq!(row.asset_id, perpetual.asset_id);
        assert_eq!(row.id, position.id);
    }

    #[test]
    fn test_info_rows_carry_the_market_values_and_explain_open_interest_and_funding() {
        let row = market_row(&Perpetual::mock(), &Asset::from_chain(Chain::HyperCore));

        let rows = info_rows(row.clone());

        assert_eq!(
            rows,
            vec![
                GemListRow::Amount {
                    title: GemListRowTitle::DailyVolume,
                    amount: row.volume_24h,
                    info: None,
                },
                GemListRow::Amount {
                    title: GemListRowTitle::OpenInterest,
                    amount: row.open_interest,
                    info: Some(GemInfoTopic::OpenInterest),
                },
                GemListRow::Amount {
                    title: GemListRowTitle::FundingApr,
                    amount: row.funding_apr,
                    info: Some(GemInfoTopic::FundingApr),
                },
            ]
        );
    }

    #[test]
    fn test_a_market_row_titles_an_unnamed_market_with_its_asset_symbol() {
        let unnamed = Perpetual { name: String::new(), ..Perpetual::mock() };
        let asset = Asset {
            symbol: "HYPE".to_string(),
            ..Asset::from_chain(Chain::HyperCore)
        };

        assert_eq!(market_row(&unnamed, &asset).title, "HYPE");
    }

    #[test]
    fn test_a_market_row_abbreviates_its_volume_and_open_interest_in_usd() {
        let row = market_row(
            &Perpetual {
                volume_24h: 1_500_000.0,
                open_interest: 5_250_000.0,
                ..Perpetual::mock()
            },
            &Asset::from_chain(Chain::HyperCore),
        );

        for number in [&row.volume_24h, &row.open_interest] {
            assert_eq!(number.unit, crate::formatted_number::GemNumberUnit::Currency { code: "USD".to_string() });
            assert_eq!(number.display, crate::formatted_number::GemNumberDisplay::Abbreviated);
        }
        assert_eq!(row.volume_24h.value, 1_500_000.0);
        assert_eq!(row.open_interest.value, 5_250_000.0);
    }

    #[test]
    fn test_a_market_row_annualizes_the_hourly_funding_as_a_signed_percent() {
        let row = |funding: f64| market_row(&Perpetual { funding, ..Perpetual::mock() }, &Asset::from_chain(Chain::HyperCore));

        assert_eq!(row(0.0013).funding_apr, GemFormattedNumber::percentage(funding_apr(0.0013), GemPercentageStyle::Signed));
        assert_eq!(row(0.0013).funding_apr.notation, crate::formatted_number::GemNumberNotation::Signed);
        assert!((row(0.0013).funding_apr.value - 11.388).abs() < 0.001);
        assert!((row(-0.0004).funding_apr.value + 3.504).abs() < 0.001);
    }

    #[test]
    fn test_an_opening_position_reads_the_same_way_a_held_one_does() {
        let opening = open_row(PerpetualDirection::Short, 5, 1_000.0);
        let held = PerpetualPosition {
            leverage: 5,
            direction: PerpetualDirection::Short,
            ..PerpetualPosition::mock()
        };
        let holding = position_row(&Perpetual::mock(), &Asset::from_chain(Chain::HyperCore), &held);

        assert_eq!(opening.position, holding.position, "a position about to open is labelled like one already open");
        assert_eq!(opening.direction_tone, holding.direction_tone);
        assert_eq!(opening.size.expect("a sized order shows its size").value, 1_000.0);
        assert_eq!(open_row(PerpetualDirection::Long, 1, 0.0).size, None, "an order with no size yet shows none");
    }

    #[test]
    fn test_a_position_row_falls_back_to_the_market_name_when_the_asset_has_no_symbol() {
        let market = Perpetual::mock();
        let mut held = PerpetualPosition::mock();
        held.leverage = 40;
        let symboled = Asset::from_chain(Chain::HyperCore);
        let unsymboled = Asset { symbol: String::new(), ..symboled.clone() };

        assert_eq!(position_row(&market, &symboled, &held).title, symboled.symbol);
        assert_eq!(position_row(&market, &unsymboled, &held).title, "BTC");
        assert_eq!(position_row(&market, &symboled, &held).position, position_text(&held.direction, 40));
    }

    #[test]
    fn test_a_position_row_carries_its_margin_its_change_and_its_label() {
        let market = Perpetual::mock();
        let held = PerpetualPosition {
            leverage: 5,
            margin_amount: 0.5432,
            pnl: -0.25,
            direction: PerpetualDirection::Short,
            ..PerpetualPosition::mock()
        };

        let row = position_row(&market, &Asset::from_chain(Chain::HyperCore), &held);

        assert_eq!(row.margin.value, 0.5432);
        assert_eq!(row.margin.unit, crate::formatted_number::GemNumberUnit::Currency { code: "USD".to_string() });
        assert_eq!(row.pnl_tone, GemValueTone::Negative, "a losing position reads as a loss on both apps");
        assert_eq!(row.direction_tone, GemValueTone::Negative, "a short reads red on both apps");
        assert!(matches!(&row.pnl, GemLocalizedText::Pnl { amount, .. } if amount.value == -0.25));
        assert_eq!(
            row.position,
            GemLocalizedText::Position {
                direction: PerpetualDirection::Short,
                leverage: GemFormattedNumber::leverage(5.0)
            },
            "the label is one value, not a direction and a leverage each app joins"
        );
    }

    #[test]
    fn test_position_action_reads_the_market_for_an_open_and_the_position_otherwise() {
        let market = Perpetual {
            identifier: "7".to_string(),
            ..Perpetual::mock()
        };
        let asset = Asset::from_chain(Chain::HyperCore);
        let mut held = PerpetualPosition::mock();
        held.direction = PerpetualDirection::Short;
        held.leverage = 5;
        held.margin_amount = 2.5;
        let open_long = GemPerpetualPositionKind::Open { direction: PerpetualDirection::Long };

        let open = position_action(&market, &asset, None, open_long.clone()).unwrap();
        let GemPerpetualPositionAction::Open { data } = &open else { panic!("expected an open") };
        assert_eq!((data.asset_index, data.leverage, &data.direction, &data.margin_type), (7, 20, &PerpetualDirection::Long, &PerpetualMarginType::Isolated));
        assert_eq!(data.base_asset.id, HYPERCORE_PERPETUAL_USDC.id);

        let reduce = position_action(&market, &asset, Some(held.clone()), GemPerpetualPositionKind::Reduce).unwrap();
        let GemPerpetualPositionAction::Reduce { data, position } = &reduce else { panic!("expected a reduce") };
        assert_eq!(
            (&data.direction, data.leverage, &data.margin_type),
            (&PerpetualDirection::Short, 5, &PerpetualMarginType::Cross),
            "a reduce keeps the position direction"
        );
        assert_eq!(*position, held);
        assert_eq!(
            perpetual_amount_type(&reduce, held.leverage),
            GemAmountType::Perpetual {
                position: GemAmountPerpetualPosition::Reduce { available: 2_500_000u64.into() },
                direction: PerpetualDirection::Short,
                price: market.price,
                leverage: 5,
                size_decimals: asset.decimals,
            }
        );
        let encoded = serde_json::to_string(&reduce).unwrap();
        assert_eq!(serde_json::from_str::<GemPerpetualPositionAction>(&encoded).unwrap(), reduce, "the action survives a route round trip");

        assert!(position_action(&market, &asset, None, GemPerpetualPositionKind::Increase).is_err(), "increasing needs a position");
        let unindexed = Perpetual {
            identifier: "BTC".to_string(),
            ..Perpetual::mock()
        };
        assert!(position_action(&unindexed, &asset, None, open_long).is_err());
    }

    #[test]
    fn test_full_reduce_uses_the_exact_close_transfer() {
        let market = Perpetual { price: 3390.0, ..Perpetual::mock() };
        let asset = Asset {
            decimals: 4,
            ..Asset::from_chain(Chain::HyperCore)
        };
        let close_data = |transfer: &GemTransferData| match transfer.input_type.get_perpetual_type().unwrap() {
            PerpetualType::Close { data } => Some(data.clone()),
            _ => None,
        };
        for direction in [PerpetualDirection::Long, PerpetualDirection::Short] {
            for margin_type in [PerpetualMarginType::Cross, PerpetualMarginType::Isolated] {
                for leverage in [1, 5] {
                    let held = PerpetualPosition {
                        direction: direction.clone(),
                        margin_type: margin_type.clone(),
                        size: 0.002,
                        size_value: 6.78,
                        leverage,
                        margin_amount: 6.0 / f64::from(leverage),
                        entry_price: 3000.0,
                        pnl: 0.78,
                        ..PerpetualPosition::mock()
                    };
                    let action = position_action(&market, &asset, Some(held.clone()), GemPerpetualPositionKind::Reduce).unwrap();
                    let close = close_transfer(&market, &asset, Some(held)).unwrap();
                    for use_max_amount in [false, true] {
                        let reduce = order_transfer(action.clone(), BigInt::from(6_000_000 / u64::from(leverage)), use_max_amount, leverage, None, None);
                        assert_eq!(
                            (close_data(&reduce), &reduce.recipient, &reduce.value, reduce.use_max_amount),
                            (close_data(&close), &close.recipient, &close.value, close.use_max_amount)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_partial_reduce_keeps_the_requested_size_even_with_the_max_flag() {
        let market = Perpetual { price: 3390.0, ..Perpetual::mock() };
        let asset = Asset {
            decimals: 4,
            ..Asset::from_chain(Chain::HyperCore)
        };
        for (direction, price) in [(PerpetualDirection::Long, "3322.2"), (PerpetualDirection::Short, "3457.8")] {
            let held = PerpetualPosition {
                direction: direction.clone(),
                margin_amount: 40.0,
                ..PerpetualPosition::mock()
            };
            let action = position_action(&market, &asset, Some(held), GemPerpetualPositionKind::Reduce).unwrap();
            for use_max_amount in [false, true] {
                let transfer = order_transfer(action.clone(), 12_000_000.into(), use_max_amount, 1, None, None);
                let reduced = match transfer.input_type.get_perpetual_type().unwrap() {
                    PerpetualType::Reduce { data } => Some((data.data.size.as_str(), data.data.price.as_str(), &data.position_direction)),
                    _ => None,
                };
                assert_eq!(reduced, Some(("0.0035", price, &direction)));
                assert_eq!(transfer.value, 12_000_000.into());
            }
        }
    }

    #[test]
    fn test_order_transfer_and_close_transfer_address_the_provider() {
        let market = Perpetual::mock();
        let asset = Asset::from_chain(Chain::HyperCore);
        let open = position_action(&market, &asset, None, GemPerpetualPositionKind::Open { direction: PerpetualDirection::Long }).unwrap();
        let transfer = order_transfer(open, BigInt::from(50_000_000), false, 8, Some(120.5), None);
        let TransactionInputType::Perpetual {
            perpetual_type: PerpetualType::Open { data },
            ..
        } = &transfer.input_type
        else {
            panic!("expected an open perpetual transfer")
        };
        let formatted = GemPerpetual::new(PerpetualProvider::Hypercore).format_price(120.5, asset.decimals);
        assert_eq!(
            (data.leverage, data.take_profit.as_deref()),
            (8, Some(formatted.as_str())),
            "the chosen leverage and a formatted trigger price go into the order"
        );
        assert_eq!(transfer.value, BigInt::from(50_000_000));
        assert_eq!(transfer.recipient, GemPerpetual::new(PerpetualProvider::Hypercore).recipient());

        let close = close_transfer(&market, &asset, Some(PerpetualPosition::mock())).unwrap();
        assert!(matches!(
            close.input_type,
            TransactionInputType::Perpetual {
                perpetual_type: PerpetualType::Close { .. },
                ..
            }
        ));
        assert!(close_transfer(&market, &asset, None).is_err());
    }

    #[test]
    fn test_every_perpetual_transfer_carries_the_chain_collateral_as_its_base_asset() {
        let market = Perpetual::mock();
        let asset = Asset::from_chain(Chain::HyperCore);
        let chain_collateral = collateral_asset_id(Chain::HyperCore).unwrap();
        let open = position_action(&market, &asset, None, GemPerpetualPositionKind::Open { direction: PerpetualDirection::Long }).unwrap();
        let transfers = [order_transfer(open, BigInt::from(50_000_000), false, 8, None, None), close_transfer(&market, &asset, Some(PerpetualPosition::mock())).unwrap()];

        for transfer in transfers {
            assert_eq!(
                transfer.input_type.get_perpetual_type().unwrap().base_asset().id,
                chain_collateral,
                "the collateral a transfer carries and the one the chain answers for must stay the same asset: the confirm load reads the balance row of the first and the price of the second"
            );
        }
    }

    #[test]
    fn test_stale_position_ids_keeps_only_positions_that_disappeared() {
        let stale = stale_position_ids(
            vec!["a".into(), "b".into()],
            &[PerpetualPosition { id: "b".into(), ..PerpetualPosition::mock() }, PerpetualPosition { id: "c".into(), ..PerpetualPosition::mock() }],
        );

        assert_eq!(stale, vec!["a".to_string()]);
    }

    #[test]
    fn test_hypercore_collateral_is_priced_at_one_dollar() {
        let price = collateral_price(Chain::HyperCore).unwrap();

        assert_eq!(price.asset_id.chain, Chain::HyperCore);
        assert_eq!(price.price, 1.0);
    }

    #[test]
    fn test_prices_outdated() {
        assert!(prices_outdated(None, 100, 5));
        assert!(prices_outdated(Some(95), 100, 5));
        assert!(!prices_outdated(Some(97), 100, 5));
    }

    #[test]
    fn test_balance_update_targets_perpetual_usdc() {
        let update = balance_update(&PerpetualBalance {
            available: 1234.5,
            reserved: 0.25,
            withdrawable: 1000.0,
        })
        .unwrap();
        assert_eq!(update.asset_id, HYPERCORE_PERPETUAL_USDC.id);
        assert!(update.is_active);
        match update.update_type {
            GemBalanceUpdateType::Perpetual { available, reserved, withdrawable } => {
                assert_eq!(available, BigUint::from(1_234_500_000u64));
                assert_eq!(reserved, BigUint::from(250_000u32));
                assert_eq!(withdrawable, BigUint::from(1_000_000_000u64));
            }
            update_type => panic!("expected a perpetual update, got {update_type:?}"),
        }
    }

    #[test]
    fn test_show_perpetuals_needs_flag_multicoin_and_hyperliquid_chain() {
        assert!(show_perpetuals(true, WalletType::Multicoin, &[Chain::Arbitrum]));
        assert!(!show_perpetuals(false, WalletType::Multicoin, &[Chain::Arbitrum]));
        assert!(!show_perpetuals(true, WalletType::Single, &[Chain::Arbitrum]));
        assert!(!show_perpetuals(true, WalletType::Multicoin, &[Chain::Bitcoin]));
    }

    #[test]
    fn test_funding_apr_annualizes_the_hourly_rate() {
        assert_eq!(funding_apr(0.0001), 0.0001 * 8760.0);
        assert_eq!(funding_apr(0.0), 0.0);
    }

    #[test]
    fn test_slippage_price_moves_against_the_trader() {
        assert_eq!(slippage_price(100.0, PerpetualDirection::Long, true, 2.0), 102.0);
        assert_eq!(slippage_price(100.0, PerpetualDirection::Short, true, 2.0), 98.0);
        assert_eq!(slippage_price(100.0, PerpetualDirection::Long, false, 2.0), 98.0);
        assert_eq!(slippage_price(100.0, PerpetualDirection::Short, false, 2.0), 102.0);
    }

    #[test]
    fn test_order_amounts_scale_the_margin_by_leverage() {
        let (size, fiat_value, margin) = order_amounts(50.0, 4, 200.0);

        assert_eq!(size, 1.0);
        assert_eq!(fiat_value, 200.0);
        assert_eq!(margin, 50.0);
    }

    #[test]
    fn test_only_reduce_closes_a_position() {
        assert!(GemPerpetualOrderAction::Open.opens_position());
        assert!(GemPerpetualOrderAction::Increase.opens_position());
        assert!(
            !GemPerpetualOrderAction::Reduce {
                position_direction: PerpetualDirection::Long
            }
            .opens_position()
        );
    }

    #[test]
    fn test_slippage_percent_defaults_to_two() {
        assert_eq!(slippage_percent(None), 2.0);
        assert_eq!(slippage_percent(Some(0.5)), 0.5);
    }

    #[test]
    fn test_confirm_details_read_the_reduce_direction_from_the_position_and_leave_a_modify_without_details() {
        let data = PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None);
        let reduce = confirm_details(&PerpetualType::Reduce {
            data: PerpetualReduceData {
                data: data.clone(),
                position_direction: PerpetualDirection::Short,
            },
        })
        .unwrap();
        assert_eq!(
            reduce.summary,
            GemPerpetualConfirmDetailsSummary {
                text: Some(GemLocalizedText::PositionChange {
                    change: GemPositionChange::Reduce,
                    direction: PerpetualDirection::Short,
                }),
                tone: GemValueTone::Neutral,
            }
        );

        let open = confirm_details(&PerpetualType::Open { data: data.clone() }).unwrap();
        assert_eq!(
            open.summary,
            GemPerpetualConfirmDetailsSummary {
                text: Some(GemLocalizedText::Position {
                    direction: PerpetualDirection::Long,
                    leverage: GemFormattedNumber::leverage(5.0),
                }),
                tone: GemValueTone::Positive,
            }
        );

        assert!(
            confirm_details(&PerpetualType::Modify {
                data: PerpetualModifyConfirmData::mock(vec![], None, None)
            })
            .is_none()
        );
    }

    #[test]
    fn test_confirm_details_sections_carry_finished_rows_and_the_trigger_prices_the_provider_sent() {
        let data = PerpetualConfirmData {
            pnl: Some(5.0),
            entry_price: Some(100.0),
            ..PerpetualConfirmData::mock(PerpetualDirection::Long, 0, Some("12.345".to_string()), None)
        };
        let sections = confirm_details(&PerpetualType::Close { data }).unwrap().sections;
        let rows = |index: usize| sections[index].rows.clone();

        assert_eq!(
            rows(0),
            vec![
                GemListRow::Label {
                    title: GemListRowTitle::Position,
                    text: GemLocalizedText::Position {
                        direction: PerpetualDirection::Long,
                        leverage: GemFormattedNumber::leverage(5.0),
                    },
                    tone: GemValueTone::Positive,
                    info: None,
                    progress: false,
                },
                GemListRow::Label {
                    title: GemListRowTitle::Pnl,
                    text: GemLocalizedText::Pnl {
                        amount: GemFormattedNumber::signed_usd(5.0),
                        percent: GemFormattedNumber::percentage(10.0, GemPercentageStyle::Signed),
                    },
                    tone: GemValueTone::Positive,
                    info: None,
                    progress: false,
                },
            ]
        );
        assert_eq!(
            rows(2),
            vec![GemListRow::Lines {
                title: GemListRowTitle::AutoClose,
                lines: vec![GemLocalizedText::TriggerOrder {
                    order: GemTriggerOrder::TakeProfit,
                    price: Some(GemFormattedNumber::usd(12.345)),
                }],
                info: None,
            }],
            "a machine decimal price is parsed once in Core, never by a locale-aware app parser"
        );
        assert_eq!(
            rows(3).first(),
            Some(&GemListRow::Amount {
                title: GemListRowTitle::MarketPrice,
                amount: GemFormattedNumber::usd(123.45),
                info: None,
            })
        );
    }

    #[test]
    fn test_the_amount_autoclose_row_names_each_trigger_and_falls_back_to_a_dash() {
        let lines = |row: GemListRow| match row {
            GemListRow::Lines { lines, .. } => lines,
            row => panic!("expected lines, got {row:?}"),
        };

        assert_eq!(
            lines(amount_autoclose_row(Some(12.345), Some(9.0))),
            vec![
                GemLocalizedText::TriggerOrder {
                    order: GemTriggerOrder::TakeProfit,
                    price: Some(GemFormattedNumber::usd(12.345)),
                },
                GemLocalizedText::TriggerOrder {
                    order: GemTriggerOrder::StopLoss,
                    price: Some(GemFormattedNumber::usd(9.0)),
                },
            ]
        );
        assert_eq!(
            lines(amount_autoclose_row(None, Some(9.0))),
            vec![GemLocalizedText::TriggerOrder {
                order: GemTriggerOrder::StopLoss,
                price: Some(GemFormattedNumber::usd(9.0)),
            }],
            "a stop loss on its own is still named"
        );
        assert_eq!(lines(amount_autoclose_row(None, None)), vec![GemLocalizedText::Text { text: EMPTY_VALUE.to_string() }]);
        assert_eq!(
            amount_autoclose_row(None, None),
            GemListRow::Lines {
                title: GemListRowTitle::AutoClose,
                lines: vec![GemLocalizedText::Text { text: EMPTY_VALUE.to_string() }],
                info: Some(GemInfoTopic::AutoClose),
            }
        );
    }

    #[test]
    fn test_confirm_details_leave_out_the_rows_the_data_has_no_value_for() {
        let sections = confirm_details(&PerpetualType::Open {
            data: PerpetualConfirmData::mock(PerpetualDirection::Short, 0, None, None),
        })
        .unwrap()
        .sections;

        assert_eq!(sections.len(), 3, "no pnl row, and no autoclose section without a trigger order");
        assert_eq!(sections[0].rows.len(), 1);
        assert_eq!(sections[2].rows.len(), 2, "no entry price row");
    }

    #[test]
    fn test_perpetual_order_keeps_the_position_action_and_prices_in_the_slippage() {
        let open = order(PerpetualProvider::Hypercore, GemPerpetualOrderInput::mock(GemPerpetualOrderAction::Open));
        let increase = order(PerpetualProvider::Hypercore, GemPerpetualOrderInput::mock(GemPerpetualOrderAction::Increase));
        let reduce = order(
            PerpetualProvider::Hypercore,
            GemPerpetualOrderInput::mock(GemPerpetualOrderAction::Reduce {
                position_direction: PerpetualDirection::Short,
            }),
        );

        let PerpetualType::Open { data } = open else { panic!("expected an open order") };
        assert_eq!(data.slippage, 2.0);
        assert_eq!(data.market_price, 100.0);
        assert_eq!(data.fiat_value, 200.0);
        assert_eq!(data.margin_amount, 50.0);
        assert!(matches!(increase, PerpetualType::Increase { .. }));
        let PerpetualType::Reduce { data: reduce } = reduce else { panic!("expected a reduce order") };
        assert_eq!(reduce.position_direction, PerpetualDirection::Short);
    }

    #[test]
    fn test_perpetual_close_order_carries_the_position_result() {
        let data = close_order(
            PerpetualProvider::Hypercore,
            GemPerpetualCloseInput {
                asset_index: 1,
                direction: PerpetualDirection::Long,
                margin_type: PerpetualMarginType::Cross,
                base_asset: Asset::mock(),
                asset: Asset::mock(),
                market_price: 100.0,
                size: -2.0,
                leverage: 4,
                pnl: 12.5,
                entry_price: 90.0,
                margin_amount: 50.0,
                slippage: None,
            },
        );

        assert_eq!(data.pnl, Some(12.5));
        assert_eq!(data.entry_price, Some(90.0));
        assert_eq!(data.fiat_value, 196.0);
    }

    #[test]
    fn test_merge_candle() {
        let candles = vec![ChartCandleStick::mock(1000, 100.0), ChartCandleStick::mock(2000, 100.0)];

        let replaced = merge_candle(candles.clone(), ChartCandleStick::mock(2000, 105.0));
        assert_eq!(replaced.len(), 2);
        assert_eq!(replaced[0].date, candles[0].date);
        assert_eq!(replaced[1].close, 105.0);

        let appended = merge_candle(candles.clone(), ChartCandleStick::mock(3000, 110.0));
        assert_eq!(appended.len(), 2);
        assert_eq!(appended[0].date, candles[1].date);
        assert_eq!(appended[1].close, 110.0);

        assert_eq!(merge_candle(candles.clone(), ChartCandleStick::mock(500, 90.0)), candles);
        assert_eq!(merge_candle(Vec::new(), ChartCandleStick::mock(500, 90.0)), Vec::new());

        let perpetual = Perpetual::mock();
        let update = ChartCandleUpdate {
            coin: symbol(&perpetual),
            interval: "30m".to_string(),
            candle: ChartCandleStick::mock(3000, 110.0),
        };
        assert_eq!(merged_candles(candles.clone(), update.clone(), &perpetual, &ChartPeriod::Day), Some(appended));
        assert_eq!(
            merged_candles(
                candles.clone(),
                ChartCandleUpdate {
                    interval: "1m".to_string(),
                    ..update.clone()
                },
                &perpetual,
                &ChartPeriod::Day
            ),
            None,
            "a candle for another interval is not this chart's"
        );
        assert_eq!(merged_candles(candles, ChartCandleUpdate { coin: "OTHER".to_string(), ..update }, &perpetual, &ChartPeriod::Day), None);
    }

    #[test]
    fn test_candle_interval() {
        assert_eq!(candle_interval(&ChartPeriod::Hour), "1m");
        assert_eq!(candle_interval(&ChartPeriod::Day), "30m");
        assert_eq!(candle_interval(&ChartPeriod::Week), "4h");
        assert_eq!(candle_interval(&ChartPeriod::Month), "12h");
        assert_eq!(candle_interval(&ChartPeriod::Year), "1w");
        assert_eq!(candle_interval(&ChartPeriod::All), "1M");
    }

    #[test]
    fn test_the_market_screen_lists_its_sections_in_order() {
        let counts = GemPerpetualMarketCounts {
            positions: 1,
            pinned: 1,
            markets: 2,
            recents: 1,
        };

        assert_eq!(
            market_sections(&counts, false, true),
            vec![GemPerpetualMarketSection::Positions, GemPerpetualMarketSection::Pinned, GemPerpetualMarketSection::Markets]
        );
        assert_eq!(
            market_sections(&counts, true, true),
            vec![GemPerpetualMarketSection::Recents, GemPerpetualMarketSection::Positions, GemPerpetualMarketSection::Pinned, GemPerpetualMarketSection::Markets],
            "an empty search query offers the recents above the rest"
        );
        assert_eq!(
            market_sections(
                &GemPerpetualMarketCounts {
                    positions: 0,
                    pinned: 0,
                    markets: 0,
                    recents: 0
                },
                true,
                false
            ),
            vec![GemPerpetualMarketSection::Empty]
        );
    }

    #[test]
    fn test_the_balance_header_totals_the_collateral_and_offers_a_withdrawal_only_with_something_to_withdraw() {
        let header = balance_header(
            Some(PerpetualBalance {
                available: 50.0,
                reserved: 25.0,
                withdrawable: 50.0,
            }),
            WalletType::Multicoin,
        );

        assert_eq!(header.total, GemFormattedNumber::usd(75.0));
        assert_eq!(header.available, GemFormattedNumber::usd(50.0));
        assert_eq!(
            header.actions,
            GemHeaderActions::Buttons {
                buttons: vec![
                    GemHeaderButton {
                        kind: GemHeaderButtonKind::Withdraw,
                        is_enabled: true
                    },
                    GemHeaderButton {
                        kind: GemHeaderButtonKind::Deposit,
                        is_enabled: true
                    }
                ]
            }
        );

        let empty = balance_header(None, WalletType::Multicoin);
        assert_eq!(empty.total, GemFormattedNumber::usd(0.0));
        assert_eq!(
            empty.actions,
            GemHeaderActions::Buttons {
                buttons: vec![
                    GemHeaderButton {
                        kind: GemHeaderButtonKind::Withdraw,
                        is_enabled: false
                    },
                    GemHeaderButton {
                        kind: GemHeaderButtonKind::Deposit,
                        is_enabled: true
                    }
                ]
            },
            "an empty balance has nothing to withdraw"
        );
        assert_eq!(balance_header(None, WalletType::View).actions, GemHeaderActions::WatchOnly);

        let withdraw = |balance: PerpetualBalance| match balance_header(Some(balance), WalletType::Multicoin).actions {
            GemHeaderActions::Buttons { buttons } => buttons.into_iter().find(|button| button.kind == GemHeaderButtonKind::Withdraw).map(|button| button.is_enabled),
            GemHeaderActions::WatchOnly => None,
        };
        let leveraged = PerpetualBalance {
            available: 50.0,
            reserved: 50.0,
            withdrawable: 0.0,
        };
        let underwater = PerpetualBalance {
            available: 0.0,
            reserved: 706.0,
            withdrawable: 305.0,
        };
        assert_eq!(withdraw(leveraged.clone()), Some(false), "margin left over above 10x leverage is not withdrawable");
        assert_eq!(withdraw(underwater.clone()), Some(true), "hyperliquid still pays out below the initial margin");
        assert_eq!(balance_total(Some(&leveraged)), GemFormattedNumber::usd(100.0));
        assert_eq!(balance_total(Some(&underwater)), GemFormattedNumber::usd(706.0));
        assert_eq!(balance_total(None), GemFormattedNumber::usd(0.0));
    }
}
