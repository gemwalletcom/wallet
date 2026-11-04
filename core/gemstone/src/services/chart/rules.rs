use std::collections::BTreeMap;
use std::ops::{Range, RangeInclusive};

use chrono::{DateTime, TimeDelta, Utc};
use primitives::{Asset, AssetLink, AssetMarket, AssetPrice, BlockExplorerLink, ChartCandleStick, ChartDateValue, ChartPeriod, ChartValue, ChartValuePercentage, Currency, PriceAlert, PriceChangeCalculator};

use super::model::{GemChartBounds, GemChartData, GemChartDateStyle, GemChartHeader, GemChartValueType};
use super::points::reduced_points;
use super::zoom::GemChartZoom;
use super::{GemChart, GemChartCurrent};
use crate::address::validate_address;
use crate::config::social::social_links;
use crate::formatted_number::GemFormattedNumber;
use crate::models::copy::{GemCopy, GemCopyKind, address_copy};
use crate::models::list::{GemInfoTopic, GemListRow, GemListRowIcon, GemListRowTitle, GemListSection, GemListSectionTitle, GemRowAction};
use crate::percentage::GemPercentageStyle;
use crate::precision::{GemCurrencyStyle, GemValueStyle};
use crate::services::price::rules::has_price;
use crate::services::price_alert::rules::displayed_price_alert_ids;

pub fn date_style(period: ChartPeriod) -> GemChartDateStyle {
    match period {
        ChartPeriod::Hour | ChartPeriod::Day => GemChartDateStyle::Relative,
        ChartPeriod::Week | ChartPeriod::Month => GemChartDateStyle::DayTime,
        ChartPeriod::Year | ChartPeriod::All => GemChartDateStyle::Day,
    }
}

const MARKET_CAP_RANK_BADGE_LIMIT: i32 = 1000;
const MIN_CHART_POINTS: usize = 2;
const RENDER_POINTS: usize = 120;
const RANGE_PADDING: f64 = 0.05;
const FLAT_LINE_PADDING: f64 = 0.01;
const FLAT_LINE_MIN_PADDING: f64 = 0.01;
pub const RIGHT_PADDING: f64 = 0.02;
const CANDLE_SIGNIFICANT_FIGURES: i32 = 5;
const CANDLE_MIN_DECIMALS: usize = 2;
const CANDLE_MAX_DECIMALS: usize = 12;

pub fn converted_values(prices: Vec<ChartValue>, rate: f64) -> Vec<ChartDateValue> {
    let mut values: Vec<ChartDateValue> = prices
        .into_iter()
        .filter_map(|price| DateTime::from_timestamp(price.timestamp as i64, 0).map(|date| ChartDateValue { date, value: price.value as f64 * rate }))
        .collect();
    values.sort_by_key(|value| value.date);
    values
}

pub fn converted_candles(candles: Vec<ChartCandleStick>, rate: f64) -> Vec<ChartCandleStick> {
    let mut candles: Vec<ChartCandleStick> = candles
        .into_iter()
        .map(|candle| ChartCandleStick {
            open: candle.open * rate,
            high: candle.high * rate,
            low: candle.low * rate,
            close: candle.close * rate,
            ..candle
        })
        .collect();
    candles.sort_by_key(|candle| candle.date);
    candles
}

pub fn candles_with_price(candles: &[ChartCandleStick], latest: Option<AssetPrice>) -> Vec<ChartCandleStick> {
    let mut candles = candles.to_vec();
    if let (Some(last), Some(latest)) = (candles.last_mut(), latest.filter(|latest| latest.price > 0.0))
        && latest.updated_at > last.date
    {
        last.close = latest.price;
        last.high = last.high.max(latest.price);
        last.low = last.low.min(latest.price);
    }
    candles
}

pub const DEFAULT_CANDLE_PERIOD: ChartPeriod = ChartPeriod::Day;

pub fn has_candle_period(period: ChartPeriod) -> bool {
    period != ChartPeriod::Hour
}

pub fn candle_price_decimals(price: f64) -> usize {
    if price <= 0.0 {
        return CANDLE_MIN_DECIMALS;
    }
    let decimals = CANDLE_SIGNIFICANT_FIGURES - 1 - price.log10().floor() as i32;
    decimals.clamp(CANDLE_MIN_DECIMALS as i32, CANDLE_MAX_DECIMALS as i32) as usize
}

pub fn base_value(values: &[ChartDateValue]) -> f64 {
    values.iter().find(|value| value.value != 0.0).or_else(|| values.first()).map_or(0.0, |value| value.value)
}

pub fn current_value(values: &[ChartDateValue], latest: Option<AssetPrice>, now: DateTime<Utc>, period: ChartPeriod, base_value: f64) -> Option<GemChartCurrent> {
    let latest = latest?;
    let is_newer = values.last().is_none_or(|last| latest.updated_at > last.date);
    is_newer.then(|| GemChartCurrent {
        date: now.max(latest.updated_at),
        value: latest.price,
        change_percentage: change_percentage(period, base_value, &latest),
    })
}

pub fn chart_with_price(chart: GemChart, price: Option<AssetPrice>, period: ChartPeriod) -> GemChart {
    match current_value(&chart.values, price, Utc::now(), period, chart.base_value) {
        Some(current) => GemChart { current: Some(current), ..chart },
        None => chart,
    }
}

fn change_percentage(period: ChartPeriod, base_value: f64, latest: &AssetPrice) -> f64 {
    match period {
        ChartPeriod::Day => latest.price_change_percentage_24h,
        ChartPeriod::Hour | ChartPeriod::Week | ChartPeriod::Month | ChartPeriod::Year | ChartPeriod::All => PriceChangeCalculator::percentage(base_value, latest.price),
    }
}

pub fn chart_sections(asset: &Asset, currency: Currency, price: Option<f64>, market: Option<&AssetMarket>, price_alerts: Option<Vec<PriceAlert>>, links: Vec<AssetLink>, contract_explorer: Option<BlockExplorerLink>) -> Vec<GemListSection> {
    let market_sections = [
        market.map(|market| market_section(market, currency.clone())).unwrap_or_default(),
        available_rows([contract_row(asset, contract_explorer)]),
        market.map(|market| supply_section(market, &asset.symbol)).unwrap_or_default(),
        market.map(|market| all_time_section(market, currency.clone())).unwrap_or_default(),
    ]
    .into_iter()
    .filter(|rows| !rows.is_empty())
    .map(GemListSection::untitled);
    price_alert_section(price, price_alerts)
        .into_iter()
        .chain(market_sections)
        .chain(
            Some(social_links(links))
                .filter(|links| !links.is_empty())
                .map(|links| GemListSection::new(GemListSectionTitle::SocialLinks, vec![GemListRow::Social { links }])),
        )
        .collect()
}

fn price_alert_section(price: Option<f64>, price_alerts: Option<Vec<PriceAlert>>) -> Option<GemListSection> {
    let price_alerts = price_alerts.filter(|_| has_price(price))?;
    let count = displayed_price_alert_ids(price_alerts).len();
    let row = match count {
        0 => GemListRow::Link {
            title: GemListRowTitle::SetPriceAlert,
            value: None,
            icon: GemListRowIcon::None,
            action: GemRowAction::SetPriceAlert,
        },
        count => GemListRow::Link {
            title: GemListRowTitle::PriceAlerts,
            value: Some(count.to_string()),
            icon: GemListRowIcon::None,
            action: GemRowAction::PriceAlerts,
        },
    };
    Some(GemListSection::untitled(vec![row]))
}

fn market_section(market: &AssetMarket, currency: Currency) -> Vec<GemListRow> {
    let value = |value: f64| GemFormattedNumber::currency(value, currency.clone(), GemCurrencyStyle::Abbreviated);
    let rank = market.market_cap_rank.filter(|rank| (1..=MARKET_CAP_RANK_BADGE_LIMIT).contains(rank));
    available_rows([
        market.market_cap.map(|market_cap| match rank {
            Some(rank) => GemListRow::ranked(GemListRowTitle::MarketCap, value(market_cap), rank),
            None => amount_row(GemListRowTitle::MarketCap, value(market_cap), None),
        }),
        market.market_cap_fdv.map(|fdv| amount_row(GemListRowTitle::FullyDilutedValuation, value(fdv), Some(GemInfoTopic::FullyDilutedValuation))),
        market.total_volume.map(|volume| amount_row(GemListRowTitle::TradingVolume, value(volume), None)),
    ])
}

fn contract_row(asset: &Asset, explorer: Option<BlockExplorerLink>) -> Option<GemListRow> {
    let contract = asset.id.contract()?;
    let chain = asset.chain();
    let copy = if validate_address(&contract, chain) {
        address_copy(chain, contract)
    } else {
        GemCopy {
            kind: GemCopyKind::Plain,
            ..address_copy(chain, contract)
        }
    };
    Some(GemListRow::identifier(GemListRowTitle::Contract, copy, explorer))
}

fn supply_section(market: &AssetMarket, symbol: &str) -> Vec<GemListRow> {
    let value = |value: f64| GemFormattedNumber::amount(value, Some(symbol.to_string()), GemValueStyle::Short);
    available_rows([
        market.circulating_supply.map(|supply| amount_row(GemListRowTitle::CirculatingSupply, value(supply), Some(GemInfoTopic::CirculatingSupply))),
        market.total_supply.map(|supply| amount_row(GemListRowTitle::TotalSupply, value(supply), Some(GemInfoTopic::TotalSupply))),
        market.max_supply.map(|supply| amount_row(GemListRowTitle::MaxSupply, value(supply), Some(GemInfoTopic::MaxSupply))),
    ])
}

fn all_time_section(market: &AssetMarket, currency: Currency) -> Vec<GemListRow> {
    let row = |title: GemListRowTitle, value: &ChartValuePercentage| GemListRow::AllTime {
        title,
        value: GemFormattedNumber::currency(value.value as f64, currency.clone(), GemCurrencyStyle::Currency),
        date: value.date,
        change: GemFormattedNumber::percentage(value.percentage as f64, GemPercentageStyle::Signed),
    };
    available_rows([
        market.all_time_high_value.as_ref().map(|value| row(GemListRowTitle::AllTimeHigh, value)),
        market.all_time_low_value.as_ref().map(|value| row(GemListRowTitle::AllTimeLow, value)),
    ])
}

fn amount_row(title: GemListRowTitle, amount: GemFormattedNumber, info: Option<GemInfoTopic>) -> GemListRow {
    GemListRow::Amount { title, amount, info }
}

fn available_rows<const N: usize>(rows: [Option<GemListRow>; N]) -> Vec<GemListRow> {
    rows.into_iter().flatten().collect()
}

fn chart_bounds(values: &[ChartDateValue], currency: Currency) -> GemChartBounds {
    let extreme = |better: fn(f64, f64) -> bool| {
        values
            .iter()
            .enumerate()
            .fold(None, |best: Option<(usize, f64)>, (index, point)| match best {
                Some((_, value)) if !better(point.value, value) => best,
                _ => Some((index, point.value)),
            })
            .unwrap_or((0, 0.0))
    };
    let (lower_index, lower) = extreme(|candidate, current| candidate < current);
    let (upper_index, upper) = extreme(|candidate, current| candidate > current);
    let range = upper - lower;
    let padding = match range == 0.0 {
        true => (lower * FLAT_LINE_PADDING).max(FLAT_LINE_MIN_PADDING),
        false => range * RANGE_PADDING,
    };
    GemChartBounds {
        lower_index,
        upper_index,
        y_min: lower - padding,
        y_max: upper + padding,
        low: GemFormattedNumber::currency(lower, currency.clone(), GemCurrencyStyle::Currency),
        high: GemFormattedNumber::currency(upper, currency, GemCurrencyStyle::Currency),
    }
}

pub fn fraction_of(span: TimeDelta, fraction: f64) -> TimeDelta {
    TimeDelta::milliseconds((span.num_milliseconds() as f64 * fraction) as i64)
}

pub fn nearest_index(dates: impl Iterator<Item = DateTime<Utc>>, window: RangeInclusive<DateTime<Utc>>, fraction: f64) -> Option<usize> {
    let date = *window.start() + fraction_of(*window.end() - *window.start(), fraction.clamp(0.0, 1.0));
    dates
        .enumerate()
        .filter(|(_, candidate)| window.contains(candidate))
        .min_by_key(|(_, candidate)| (*candidate - date).abs())
        .map(|(index, _)| index)
}

pub fn zoomed_chart(data: GemChartData, zoom: GemChartZoom) -> GemChartData {
    let zoom = zoom.clamped(data.values.len());
    let (start, end) = zoom.window(data.start, data.end).into_inner();
    let window = start..=(end + fraction_of(end - start, RIGHT_PADDING));
    let visible = &data.values[visible_range(&data.values, &window)];
    let visible_bounds = chart_bounds(visible, data.currency.clone());
    let points = reduced_points(&data.values, render_points(zoom));
    let points_range = visible_range(&points, &window);
    let values: Vec<ChartDateValue> = points[points_range.start.saturating_sub(1)..(points_range.end + 1).min(points.len())]
        .iter()
        .chain([visible_bounds.lower_index, visible_bounds.upper_index].into_iter().filter_map(|index| visible.get(index)))
        .map(|value| (value.date, value.clone()))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect();
    let values_range = match visible_range(&values, &window) {
        range if range.is_empty() => 0..values.len(),
        range => range,
    };
    let bounds = chart_bounds(&values[values_range.clone()], data.currency.clone());
    GemChartData {
        bounds: GemChartBounds {
            lower_index: bounds.lower_index + values_range.start,
            upper_index: bounds.upper_index + values_range.start,
            ..bounds
        },
        values,
        start: *window.start(),
        end: *window.end(),
        is_zoomed: zoom.is_zoomed(),
        ..data
    }
}

fn visible_range(values: &[ChartDateValue], window: &RangeInclusive<DateTime<Utc>>) -> Range<usize> {
    values.partition_point(|value| value.date < *window.start())..values.partition_point(|value| value.date <= *window.end())
}

fn render_points(zoom: GemChartZoom) -> usize {
    RENDER_POINTS << zoom.scale.log2().floor() as usize
}

fn chart_data(values: Vec<ChartDateValue>, value_type: GemChartValueType, base: f64, shows_secondary_value: bool, period: ChartPeriod, currency: Currency) -> Option<GemChartData> {
    if values.len() < MIN_CHART_POINTS || values.iter().any(|value| !value.value.is_finite()) {
        return None;
    }
    if matches!(value_type, GemChartValueType::PriceChange) && values.windows(2).all(|pair| pair[0].value == pair[1].value) {
        return None;
    }
    Some(GemChartData {
        value_type,
        base,
        shows_secondary_value,
        bounds: chart_bounds(&values, currency.clone()),
        currency,
        start: values.first()?.date,
        end: values.last()?.date,
        is_zoomed: false,
        values,
        header: None,
        date_style: date_style(period),
    })
}

pub fn price_chart_data(chart: GemChart, period: ChartPeriod, currency: Currency) -> Option<GemChartData> {
    let base = chart.base_value;
    let current = chart.current;
    let values: Vec<ChartDateValue> = chart.values.into_iter().chain(current.as_ref().map(|current| ChartDateValue { date: current.date, value: current.value })).collect();
    let data = chart_data(values, GemChartValueType::Price, base, false, period, currency)?;
    let header = match &current {
        Some(current) => header(&data, current.value, Some(current.change_percentage)),
        None => header(&data, data.values.last()?.value, None),
    };
    Some(GemChartData { header: Some(header), ..data })
}

pub fn change_chart_data(values: Vec<ChartDateValue>, shows_secondary_value: bool, period: ChartPeriod, currency: Currency) -> Option<GemChartData> {
    let base = values.first()?.value;
    let data = chart_data(values, GemChartValueType::PriceChange, base, shows_secondary_value, period, currency)?;
    let header = header(&data, data.values.last()?.value, None);
    Some(GemChartData { header: Some(header), ..data })
}

pub fn header(data: &GemChartData, value: f64, change_percentage: Option<f64>) -> GemChartHeader {
    series_header(data.value_type, data.base, data.shows_secondary_value, &data.currency, value, change_percentage)
}

pub fn series_header(value_type: GemChartValueType, base: f64, shows_secondary_value: bool, currency: &Currency, value: f64, change_percentage: Option<f64>) -> GemChartHeader {
    let change_percentage = change_percentage.unwrap_or_else(|| PriceChangeCalculator::percentage(base, value));
    let (display_value, secondary_value) = match value_type {
        GemChartValueType::Price => (value, None),
        GemChartValueType::PriceChange => (value - base, shows_secondary_value.then_some(value)),
    };
    let shows_change = display_value != 0.0
        && match value_type {
            GemChartValueType::Price => true,
            GemChartValueType::PriceChange => secondary_value.is_some() && change_percentage != 0.0,
        };
    let price = |value: f64| GemFormattedNumber::currency(value, currency.clone(), GemCurrencyStyle::Currency);
    GemChartHeader {
        value: match value_type {
            GemChartValueType::Price => price(display_value),
            GemChartValueType::PriceChange => GemFormattedNumber::signed_currency(display_value, currency.clone(), GemCurrencyStyle::Currency),
        },
        secondary_value: secondary_value.map(price),
        change: shows_change.then(|| match value_type {
            GemChartValueType::Price => GemFormattedNumber::percentage(change_percentage, GemPercentageStyle::Signed),
            GemChartValueType::PriceChange => GemFormattedNumber::percentage(change_percentage, GemPercentageStyle::Unsigned).in_parentheses().toned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usd(value: f64) -> GemFormattedNumber {
        GemFormattedNumber::currency(value, Currency::USD, GemCurrencyStyle::Abbreviated)
    }

    fn supply(value: f64, symbol: &str) -> GemFormattedNumber {
        GemFormattedNumber::amount(value, Some(symbol.to_string()), GemValueStyle::Short)
    }

    fn set_price_alert() -> GemListSection {
        GemListSection::untitled(vec![GemListRow::Link {
            title: GemListRowTitle::SetPriceAlert,
            value: None,
            icon: GemListRowIcon::None,
            action: GemRowAction::SetPriceAlert,
        }])
    }
    use crate::formatted_number::GemValueTone;
    use primitives::{AssetId, ChartValuePercentage, LinkType, PriceAlertDirection, asset_constants::STELLAR_USDC_ASSET_ID, currency::Currency};

    #[test]
    fn test_a_chart_date_reads_by_how_long_the_period_is() {
        assert_eq!(date_style(ChartPeriod::Hour), GemChartDateStyle::Relative);
        assert_eq!(date_style(ChartPeriod::Day), GemChartDateStyle::Relative);
        assert_eq!(date_style(ChartPeriod::Week), GemChartDateStyle::DayTime);
        assert_eq!(date_style(ChartPeriod::Month), GemChartDateStyle::DayTime);
        assert_eq!(date_style(ChartPeriod::Year), GemChartDateStyle::Day, "a year of points needs no clock");
        assert_eq!(date_style(ChartPeriod::All), GemChartDateStyle::Day);
    }

    #[test]
    fn test_nearest_index() {
        let at = |seconds: i64| DateTime::from_timestamp(seconds, 0).unwrap();
        let dates = [at(0), at(60), at(120)];

        assert_eq!(nearest_index(dates.into_iter(), at(0)..=at(120), 0.4), Some(1));
        assert_eq!(nearest_index(dates.into_iter(), at(0)..=at(120), 4.0), Some(2), "a finger past the edge picks the edge point");
        assert_eq!(nearest_index(dates.into_iter(), at(30)..=at(120), 0.0), Some(1), "a point outside the window is never picked");
    }

    #[test]
    fn test_zoomed_chart() {
        let at = |milliseconds: i64| DateTime::from_timestamp_millis(milliseconds).unwrap();
        let data = price_chart_data(GemChart::mock(ChartDateValue::mock_series(600)), ChartPeriod::Day, Currency::USD).unwrap();
        let whole = zoomed_chart(data.clone(), GemChartZoom::default());
        let zoomed = zoomed_chart(data, GemChartZoom { scale: 3.0, offset: 0.25 });
        let last = zoomed.values.len() - 1;
        let gapped: Vec<ChartDateValue> = (0..300).map(|second| ChartDateValue::mock(second, 100.0)).chain((0..300).map(|second| ChartDateValue::mock(100_000 + second, 200.0))).collect();
        let gap = zoomed_chart(price_chart_data(GemChart::mock(gapped), ChartPeriod::Day, Currency::USD).unwrap(), GemChartZoom { scale: 40.0, offset: 0.5 });

        assert_eq!((whole.start, whole.end), (at(0), at(610_980)), "the whole period plus room after the newest point");
        assert_eq!((zoomed.start, zoomed.end), (at(249_584), at(453_243)));
        assert_eq!(
            (zoomed.values[zoomed.bounds.lower_index].value, zoomed.values[zoomed.bounds.upper_index].value),
            (250.0, 453.0),
            "the range fits the points inside the window"
        );
        assert_eq!((zoomed.selection(0), zoomed.selection(last)), (None, None), "the points drawn past either edge cannot be selected");
        assert_eq!(zoomed.index_at(1.2), Some(last - 1));
        assert_eq!((whole.is_zoomed, zoomed.is_zoomed), (false, true));
        assert_eq!(
            (gap.values[gap.bounds.lower_index].value, gap.values[gap.bounds.upper_index].value),
            (100.0, 200.0),
            "a window inside a gap fits the line crossing it"
        );
    }

    #[test]
    fn test_render_points() {
        assert_eq!(render_points(GemChartZoom { scale: 1.9, offset: 0.0 }), 120, "detail changes only at whole doublings, so points stay put while pinching");
        assert_eq!(render_points(GemChartZoom { scale: 4.0, offset: 0.0 }), 480);
    }

    #[test]
    fn test_price_chart_data_needs_two_points() {
        assert_eq!(price_chart_data(GemChart::mock(vec![ChartDateValue::mock(1, 100.0)]), ChartPeriod::Day, Currency::USD), None);
        assert_eq!(price_chart_data(GemChart::mock(vec![]), ChartPeriod::Day, Currency::USD), None);
    }

    #[test]
    fn test_chart_data_rejects_non_finite_values() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let values = vec![ChartDateValue::mock(1, 100.0), ChartDateValue::mock(2, value)];

            assert_eq!(price_chart_data(GemChart::mock(values.clone()), ChartPeriod::Day, Currency::USD), None);
            assert_eq!(change_chart_data(values, true, ChartPeriod::Day, Currency::USD), None);
        }
    }

    #[test]
    fn test_price_chart_data_appends_the_current_point_after_history() {
        let current = GemChartCurrent {
            date: DateTime::from_timestamp(2_000, 0).unwrap(),
            value: 200.0,
            change_percentage: 4.2,
        };
        let chart = GemChart {
            has_candles: false,
            current: Some(current),
            ..GemChart::mock(vec![ChartDateValue::mock(1_000, 100.0)])
        };
        let data = price_chart_data(chart, ChartPeriod::Day, Currency::USD).expect("data");

        assert_eq!(data.value_type, GemChartValueType::Price);
        assert_eq!(data.base, 100.0);
        assert_eq!(data.values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![100.0, 200.0]);
        assert_eq!(
            data.header,
            Some(GemChartHeader {
                value: GemFormattedNumber::currency(200.0, Currency::USD, GemCurrencyStyle::Currency),
                secondary_value: None,
                change: Some(GemFormattedNumber::percentage(4.2, GemPercentageStyle::Signed)),
            })
        );
    }

    #[test]
    fn test_price_chart_data_header_falls_back_to_the_last_point() {
        let data = price_chart_data(GemChart::mock(vec![ChartDateValue::mock(1, 100.0), ChartDateValue::mock(2, 150.0)]), ChartPeriod::Day, Currency::USD).expect("data");

        assert_eq!(
            data.header,
            Some(GemChartHeader {
                value: GemFormattedNumber::currency(150.0, Currency::USD, GemCurrencyStyle::Currency),
                secondary_value: None,
                change: Some(GemFormattedNumber::percentage(50.0, GemPercentageStyle::Signed)),
            })
        );
    }

    #[test]
    fn test_change_chart_data_needs_a_series_that_moves() {
        assert_eq!(
            change_chart_data(vec![ChartDateValue::mock(1, 5.0), ChartDateValue::mock(2, 5.0), ChartDateValue::mock(3, 5.0)], true, ChartPeriod::Day, Currency::USD),
            None
        );
        assert_eq!(change_chart_data(vec![ChartDateValue::mock(1, 5.0)], true, ChartPeriod::Day, Currency::USD), None);
    }

    #[test]
    fn test_change_chart_data_header_is_the_distance_from_the_first_value() {
        let data = change_chart_data(vec![ChartDateValue::mock(1, 10.0), ChartDateValue::mock(2, 12.0), ChartDateValue::mock(3, 15.0)], true, ChartPeriod::Day, Currency::USD).expect("data");

        assert_eq!(data.value_type, GemChartValueType::PriceChange);
        assert_eq!(data.base, 10.0);
        assert_eq!(
            data.header,
            Some(GemChartHeader {
                value: GemFormattedNumber::signed_currency(5.0, Currency::USD, GemCurrencyStyle::Currency),
                secondary_value: Some(GemFormattedNumber::currency(15.0, Currency::USD, GemCurrencyStyle::Currency)),
                change: Some(GemFormattedNumber::percentage(50.0, GemPercentageStyle::Unsigned).in_parentheses().toned()),
            })
        );
        assert_eq!(data.selection(1).map(|selection| (selection.header.value.value, selection.date)), Some((2.0, ChartDateValue::mock(2, 12.0).date)));
        assert_eq!(data.selection(3), None, "a selection past the last point answers nothing");
        assert_eq!(data.date_style, GemChartDateStyle::Relative);
    }

    #[test]
    fn test_change_chart_data_hides_the_percentage_without_a_secondary_value() {
        let values = vec![ChartDateValue::mock(1, 10.0), ChartDateValue::mock(2, 12.0)];

        assert_eq!(change_chart_data(values.clone(), false, ChartPeriod::Day, Currency::USD).unwrap().header.unwrap().change, None);
        assert_eq!(
            change_chart_data(values, true, ChartPeriod::Day, Currency::USD).unwrap().header.unwrap().change,
            Some(GemFormattedNumber::percentage(20.0, GemPercentageStyle::Unsigned).in_parentheses().toned())
        );
    }

    #[test]
    fn test_a_price_headline_is_plain_and_a_change_headline_carries_its_direction() {
        let price = price_chart_data(GemChart::mock(vec![ChartDateValue::mock(1, 100.0), ChartDateValue::mock(2, 90.0)]), ChartPeriod::Day, Currency::USD).expect("data");
        let price_header = price.header.expect("header");
        assert_eq!(price_header.value.tone, GemValueTone::Plain);
        assert_eq!(price_header.change.expect("change").tone, GemValueTone::Negative);

        let change = change_chart_data(vec![ChartDateValue::mock(1, 100.0), ChartDateValue::mock(2, 90.0)], true, ChartPeriod::Day, Currency::USD).expect("data");
        let change_header = change.header.expect("header");
        assert_eq!(change_header.value.tone, GemValueTone::Negative);
        assert_eq!(change_header.change.expect("change").tone, GemValueTone::Negative, "the parenthesised percentage follows the headline");
    }

    #[test]
    fn test_header_hides_the_percentage_of_a_zero_value() {
        assert_eq!(super::super::candlestick_header(100.0, 0.0, 2, &Currency::USD).change, None);
        assert_eq!(super::super::candlestick_header(100.0, 150.0, 2, &Currency::USD).change, Some(GemFormattedNumber::percentage(50.0, GemPercentageStyle::Signed)));
    }

    #[test]
    fn test_converted_values_apply_rate_and_sort() {
        let values = converted_values(vec![ChartValue { timestamp: 20, value: 2.0 }, ChartValue { timestamp: 10, value: 1.5 }], 2.0);
        assert_eq!(values.iter().map(|value| value.date.timestamp()).collect::<Vec<_>>(), vec![10, 20]);
        assert_eq!(values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![3.0, 4.0]);
    }

    #[test]
    fn test_current_value_is_only_a_price_newer_than_the_chart() {
        let points = [ChartDateValue::mock(10, 1.0), ChartDateValue::mock(20, 1.0)];
        let newer = AssetPrice::new(AssetId::from_chain(primitives::Chain::Bitcoin), 9.0, 4.2, DateTime::from_timestamp(30, 0).unwrap());
        let same_age = AssetPrice {
            updated_at: DateTime::from_timestamp(20, 0).unwrap(),
            ..newer.clone()
        };
        let now = DateTime::from_timestamp(500, 0).unwrap();

        let current = current_value(&points, Some(newer.clone()), now, ChartPeriod::Day, 1.0).expect("current");
        assert_eq!(current.value, 9.0);
        assert_eq!(current.date, now);
        assert_eq!(current.change_percentage, 4.2);
        assert_eq!(current_value(&points, Some(newer.clone()), now, ChartPeriod::Week, 3.0).unwrap().change_percentage, 200.0);
        assert_eq!(current_value(&points, Some(newer.clone()), now, ChartPeriod::Week, 0.0).unwrap().change_percentage, 0.0);

        assert_eq!(
            current_value(&points, Some(newer.clone()), DateTime::from_timestamp(15, 0).unwrap(), ChartPeriod::Day, 1.0).unwrap().date,
            newer.updated_at,
            "a device clock behind the server still places the live price after the last chart point"
        );
        assert_eq!(current_value(&points, Some(same_age.clone()), now, ChartPeriod::Day, 1.0), None);
        assert_eq!(current_value(&points, None, now, ChartPeriod::Day, 1.0), None);
        assert!(current_value(&[], Some(same_age), now, ChartPeriod::Day, 0.0).is_some());
    }

    #[test]
    fn test_base_value_is_the_first_non_zero_value() {
        assert_eq!(base_value(&[ChartDateValue::mock(0, 0.0), ChartDateValue::mock(0, 100.0), ChartDateValue::mock(0, 200.0)]), 100.0);
        assert_eq!(base_value(&[ChartDateValue::mock(0, 50.0), ChartDateValue::mock(0, 100.0)]), 50.0);
        assert_eq!(base_value(&[ChartDateValue::mock(0, 0.0)]), 0.0);
        assert_eq!(base_value(&[]), 0.0);
    }

    #[test]
    fn test_chart_bounds_pad_the_range_and_point_at_the_first_extremes() {
        let points = |values: &[f64]| values.iter().map(|value| ChartDateValue { date: Utc::now(), value: *value }).collect::<Vec<_>>();

        let bounds = chart_bounds(&points(&[100.0, 150.0, 80.0, 120.0, 80.0]), Currency::USD);
        assert_eq!((bounds.lower_index, bounds.upper_index), (2, 1));
        assert_eq!((bounds.low.value, bounds.high.value), (80.0, 150.0), "the labels name the extremes, not the padded range");
        assert!((bounds.y_min - 76.5).abs() < 1e-9 && (bounds.y_max - 153.5).abs() < 1e-9);

        let flat = chart_bounds(&points(&[5.0, 5.0]), Currency::USD);
        assert!((flat.y_min - 4.95).abs() < 1e-9 && (flat.y_max - 5.05).abs() < 1e-9, "a flat line pads by one percent");

        let zero = chart_bounds(&points(&[0.0, 0.0]), Currency::USD);
        assert!((zero.y_min + 0.01).abs() < 1e-9 && (zero.y_max - 0.01).abs() < 1e-9, "a flat line at zero keeps a minimum range");

        let negative = chart_bounds(&points(&[-2.0, -2.0]), Currency::USD);
        assert!((negative.y_min + 2.01).abs() < 1e-9 && (negative.y_max + 1.99).abs() < 1e-9);
    }

    fn amount(title: GemListRowTitle, amount: GemFormattedNumber, info: Option<GemInfoTopic>) -> GemListRow {
        GemListRow::Amount { title, amount, info }
    }

    fn all_time(title: GemListRowTitle, value: ChartValuePercentage) -> GemListRow {
        GemListRow::AllTime {
            title,
            value: GemFormattedNumber::currency(value.value as f64, Currency::USD, GemCurrencyStyle::Currency),
            date: value.date,
            change: GemFormattedNumber::percentage(value.percentage as f64, GemPercentageStyle::Signed),
        }
    }

    fn contract(token: &Asset, explorer: Option<BlockExplorerLink>) -> GemListRow {
        GemListRow::identifier(GemListRowTitle::Contract, address_copy(token.chain(), token.id.token_id.clone().unwrap()), explorer)
    }

    #[test]
    fn test_chart_sections_group_the_market_rows_core_has() {
        let token = Asset::mock_ethereum_usdc();
        let market = AssetMarket::mock();
        let links = vec![AssetLink::new("https://example.com", LinkType::Website)];

        assert_eq!(
            chart_sections(&token, Currency::USD, Some(1.0), Some(&market), Some(vec![]), links.clone(), Some(BlockExplorerLink::mock())),
            vec![
                set_price_alert(),
                GemListSection::untitled(vec![
                    GemListRow::Ranked {
                        title: GemListRowTitle::MarketCap,
                        amount: usd(100.0),
                        tag: "#1".to_string()
                    },
                    amount(GemListRowTitle::FullyDilutedValuation, usd(120.0), Some(GemInfoTopic::FullyDilutedValuation)),
                    amount(GemListRowTitle::TradingVolume, usd(10.0), None),
                ]),
                GemListSection::untitled(vec![contract(&token, Some(BlockExplorerLink::mock()))]),
                GemListSection::untitled(vec![
                    amount(GemListRowTitle::CirculatingSupply, supply(50.0, &token.symbol), Some(GemInfoTopic::CirculatingSupply)),
                    amount(GemListRowTitle::TotalSupply, supply(60.0, &token.symbol), Some(GemInfoTopic::TotalSupply)),
                    amount(GemListRowTitle::MaxSupply, supply(21.0, &token.symbol), Some(GemInfoTopic::MaxSupply)),
                ]),
                GemListSection::untitled(vec![
                    all_time(GemListRowTitle::AllTimeHigh, ChartValuePercentage::mock()),
                    all_time(GemListRowTitle::AllTimeLow, ChartValuePercentage::mock_low())
                ]),
                GemListSection::new(GemListSectionTitle::SocialLinks, vec![GemListRow::Social { links: social_links(links) }]),
            ]
        );
    }

    #[test]
    fn test_chart_sections_skip_missing_values_and_empty_sections() {
        let sections = chart_sections(&Asset::mock(), Currency::USD, None, Some(&AssetMarket::mock_partial()), Some(vec![]), vec![], None);

        assert_eq!(
            sections,
            vec![
                GemListSection::untitled(vec![amount(GemListRowTitle::FullyDilutedValuation, usd(120.0), Some(GemInfoTopic::FullyDilutedValuation))]),
                GemListSection::untitled(vec![
                    amount(GemListRowTitle::CirculatingSupply, supply(50.0, &Asset::mock().symbol), Some(GemInfoTopic::CirculatingSupply)),
                    amount(GemListRowTitle::MaxSupply, supply(21.0, &Asset::mock().symbol), Some(GemInfoTopic::MaxSupply)),
                ]),
                GemListSection::untitled(vec![all_time(GemListRowTitle::AllTimeHigh, ChartValuePercentage::mock())]),
            ]
        );
    }

    #[test]
    fn test_chart_sections_rank_badge_limit() {
        let rank = |rank: i32| {
            chart_sections(&Asset::mock(), Currency::USD, None, Some(&AssetMarket::mock_with_rank(rank)), Some(vec![]), vec![], None)
                .remove(0)
                .rows
                .remove(0)
        };

        assert_eq!(
            rank(MARKET_CAP_RANK_BADGE_LIMIT),
            GemListRow::Ranked {
                title: GemListRowTitle::MarketCap,
                amount: usd(100.0),
                tag: "#1000".to_string()
            }
        );
        assert_eq!(rank(MARKET_CAP_RANK_BADGE_LIMIT + 1), amount(GemListRowTitle::MarketCap, usd(100.0), None));
        assert_eq!(rank(0), amount(GemListRowTitle::MarketCap, usd(100.0), None));
    }

    #[test]
    fn test_chart_sections_without_market_keep_contract() {
        let token = Asset::mock_ethereum_usdc();

        assert_eq!(chart_sections(&token, Currency::USD, None, None, Some(vec![]), vec![], None), vec![GemListSection::untitled(vec![contract(&token, None)])]);
    }

    #[test]
    fn test_chart_sections_copy_a_contract_that_is_not_an_address_as_plain_text() {
        let token = Asset {
            id: STELLAR_USDC_ASSET_ID.clone(),
            ..Asset::mock()
        };
        let copy = GemCopy {
            display: "USDC:...4KZVN".to_string(),
            ..GemCopy::plain("USDC:GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN".to_string())
        };

        assert_eq!(
            chart_sections(&token, Currency::USD, None, None, Some(vec![]), vec![], None),
            vec![GemListSection::untitled(vec![GemListRow::identifier(GemListRowTitle::Contract, copy, None)])]
        );
    }

    #[test]
    fn test_chart_sections_offer_price_alerts_only_with_a_price() {
        let asset = Asset::mock();
        let auto = PriceAlert::new_auto(asset.id.clone(), Currency::USD);
        let mut notified = PriceAlert::new_price(asset.id.clone(), Currency::USD, 120.0, PriceAlertDirection::Up);
        notified.last_notified_at = Some(Utc::now());
        let first = |price: Option<f64>, alerts: Vec<PriceAlert>| chart_sections(&asset, Currency::USD, price, None, Some(alerts), vec![], None).into_iter().next();

        assert_eq!(first(Some(1.0), vec![]), Some(set_price_alert()));
        assert_eq!(
            first(Some(1.0), vec![auto.clone(), notified.clone()]),
            Some(GemListSection::untitled(vec![GemListRow::Link {
                title: GemListRowTitle::PriceAlerts,
                value: Some("1".to_string()),
                icon: GemListRowIcon::None,
                action: GemRowAction::PriceAlerts,
            }]))
        );
        assert_eq!(first(Some(1.0), vec![notified]), Some(set_price_alert()));
        assert_eq!(first(Some(0.0), vec![auto.clone()]), None);
        assert_eq!(first(None, vec![auto]), None);
        assert_eq!(first(None, vec![]), None);
    }

    #[test]
    fn test_chart_sections_offer_no_price_alerts_when_the_app_cannot_push() {
        assert_eq!(chart_sections(&Asset::mock(), Currency::USD, Some(1.0), None, None, vec![], None), vec![]);
    }

    fn candle(open: f64, high: f64, low: f64, close: f64) -> ChartCandleStick {
        ChartCandleStick {
            date: DateTime::from_timestamp(1_000, 0).unwrap(),
            open,
            high,
            low,
            close,
            volume: 0.0,
        }
    }

    #[test]
    fn test_converted_candles_converts_every_price_and_sorts_by_date() {
        let later = ChartCandleStick {
            date: DateTime::from_timestamp(2_000, 0).unwrap(),
            ..candle(1.0, 2.0, 0.5, 1.5)
        };

        let candles = converted_candles(vec![later, candle(2.0, 4.0, 1.0, 3.0)], 0.5);

        assert_eq!(candles[0], candle(1.0, 2.0, 0.5, 1.5));
        assert_eq!(candles[1].close, 0.75);
    }

    #[test]
    fn test_candles_with_price_moves_the_last_candle_to_a_newer_price() {
        let candles = vec![candle(1.0, 2.0, 0.5, 1.5)];
        let price = |price: f64, seconds: i64| Some(AssetPrice::new(AssetId::from_chain(primitives::Chain::Bitcoin), price, 0.0, DateTime::from_timestamp(seconds, 0).unwrap()));

        assert_eq!(candles_with_price(&candles, price(3.0, 2_000)), vec![candle(1.0, 3.0, 0.5, 3.0)]);
        assert_eq!(candles_with_price(&candles, price(0.25, 2_000)), vec![candle(1.0, 2.0, 0.25, 0.25)]);
        assert_eq!(candles_with_price(&candles, price(3.0, 500)), candles, "an older price leaves the candle alone");
        assert_eq!(candles_with_price(&candles, price(0.0, 2_000)), candles, "no price leaves the candle alone");
        assert_eq!(candles_with_price(&candles, None), candles);
    }

    #[test]
    fn test_candle_price_decimals_keeps_five_significant_figures() {
        assert_eq!(candle_price_decimals(80_812.2), 2);
        assert_eq!(candle_price_decimals(1.4251), 4);
        assert_eq!(candle_price_decimals(0.0001234), 8);
        assert_eq!(candle_price_decimals(0.000000000001), 12);
        assert_eq!(candle_price_decimals(0.0), 2);
    }
}
