use chrono::{DateTime, Utc};
use primitives::{Asset, AssetLink, AssetMarket, AssetPrice, BlockExplorerLink, ChartDateValue, ChartPeriod, ChartValue, ChartValuePercentage, Currency, PriceAlert, PriceChangeCalculator};

use super::model::{GemChartBounds, GemChartData, GemChartDateStyle, GemChartHeader, GemChartValueType};
use super::{GemChart, GemChartCurrent};
use crate::config::social::social_links;
use crate::formatted_number::GemFormattedNumber;
use crate::models::copy::address_copy;
use crate::models::list::{GemInfoTopic, GemListRow, GemListRowIcon, GemListRowTitle, GemListSection, GemListSectionFooter, GemListSectionTitle, GemRowAction};
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
const RANGE_PADDING: f64 = 0.05;
const FLAT_LINE_PADDING: f64 = 0.01;
const FLAT_LINE_MIN_PADDING: f64 = 0.01;

pub fn converted_values(prices: Vec<ChartValue>, rate: f64) -> Vec<ChartDateValue> {
    let mut values: Vec<ChartDateValue> = prices
        .into_iter()
        .filter_map(|price| DateTime::from_timestamp(price.timestamp as i64, 0).map(|date| ChartDateValue { date, value: price.value as f64 * rate }))
        .collect();
    values.sort_by_key(|value| value.date);
    values
}

pub fn base_value(values: &[ChartDateValue]) -> f64 {
    values.iter().find(|value| value.value != 0.0).or(values.first()).map_or(0.0, |value| value.value)
}

pub fn current_value(values: &[ChartDateValue], latest: Option<AssetPrice>, now: DateTime<Utc>, period: ChartPeriod, base_value: f64) -> Option<GemChartCurrent> {
    let latest = latest?;
    let is_newer = values.last().is_none_or(|last| latest.updated_at > last.date);
    is_newer.then(|| GemChartCurrent {
        date: now,
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

pub fn chart_sections(asset: &Asset, currency: Currency, price: Option<f64>, market: Option<&AssetMarket>, price_alerts: Vec<PriceAlert>, links: Vec<AssetLink>, contract_explorer: Option<BlockExplorerLink>) -> Vec<GemListSection> {
    let market_sections = [
        market.map(|market| market_section(market, currency.clone())).unwrap_or_default(),
        available_rows([contract_row(asset, contract_explorer)]),
        market.map(|market| supply_section(market, &asset.symbol)).unwrap_or_default(),
        market.map(|market| all_time_section(market, currency.clone())).unwrap_or_default(),
    ]
    .into_iter()
    .filter(|rows| !rows.is_empty())
    .map(|rows| section(GemListSectionTitle::None, rows));
    price_alert_section(price, price_alerts)
        .into_iter()
        .chain(market_sections)
        .chain(
            Some(social_links(links))
                .filter(|links| !links.is_empty())
                .map(|links| section(GemListSectionTitle::SocialLinks, vec![GemListRow::Social { links }])),
        )
        .collect()
}

fn section(title: GemListSectionTitle, rows: Vec<GemListRow>) -> GemListSection {
    GemListSection {
        title,
        footer: GemListSectionFooter::None,
        rows,
    }
}

fn price_alert_section(price: Option<f64>, price_alerts: Vec<PriceAlert>) -> Option<GemListSection> {
    if !has_price(price) {
        return None;
    }
    let count = displayed_price_alert_ids(price_alerts).len() as u32;
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
    Some(section(GemListSectionTitle::None, vec![row]))
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
    let token_id = asset.id.token_id.clone()?;
    Some(GemListRow::identifier(GemListRowTitle::Contract, address_copy(asset.chain(), token_id), explorer))
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

pub fn chart_bounds(values: &[ChartDateValue], currency: Currency) -> GemChartBounds {
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
        lower_index: lower_index as u32,
        upper_index: upper_index as u32,
        y_min: lower - padding,
        y_max: upper + padding,
        low: GemFormattedNumber::currency(lower, currency.clone(), GemCurrencyStyle::Currency),
        high: GemFormattedNumber::currency(upper, currency, GemCurrencyStyle::Currency),
    }
}

pub fn price_chart_data(chart: GemChart, period: ChartPeriod, currency: Currency) -> Option<GemChartData> {
    let base = chart.base_value;
    let current = chart.current;
    let values: Vec<ChartDateValue> = chart.values.into_iter().chain(current.as_ref().map(|current| ChartDateValue { date: current.date, value: current.value })).collect();
    if values.len() < MIN_CHART_POINTS {
        return None;
    }
    let last = values.last()?.value;
    let data = GemChartData {
        value_type: GemChartValueType::Price,
        base,
        shows_secondary_value: false,
        bounds: chart_bounds(&values, currency.clone()),
        currency,
        values,
        header: None,
        date_style: date_style(period),
    };
    let header = match &current {
        Some(current) => header(&data, current.value, Some(current.change_percentage)),
        None => header(&data, last, None),
    };
    Some(GemChartData { header: Some(header), ..data })
}

pub fn change_chart_data(values: Vec<ChartDateValue>, shows_secondary_value: bool, period: ChartPeriod, currency: Currency) -> Option<GemChartData> {
    if values.len() < MIN_CHART_POINTS || !has_variation(&values) {
        return None;
    }
    let base = values.first()?.value;
    let last = values.last()?.value;
    let data = GemChartData {
        value_type: GemChartValueType::PriceChange,
        base,
        shows_secondary_value,
        bounds: chart_bounds(&values, currency.clone()),
        currency,
        values,
        header: None,
        date_style: date_style(period),
    };
    let header = header(&data, last, None);
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

fn has_variation(values: &[ChartDateValue]) -> bool {
    let first = values.first().map(|value| value.value);
    values.iter().any(|value| Some(value.value) != first)
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
        section(
            GemListSectionTitle::None,
            vec![GemListRow::Link {
                title: GemListRowTitle::SetPriceAlert,
                value: None,
                icon: GemListRowIcon::None,
                action: GemRowAction::SetPriceAlert,
            }],
        )
    }
    use crate::formatted_number::GemValueTone;
    use primitives::{AssetId, ChartValuePercentage, LinkType, PriceAlertDirection, currency::Currency};

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
    fn test_price_chart_data_needs_two_points() {
        assert_eq!(price_chart_data(GemChart::mock(vec![ChartDateValue::mock(1, 100.0)]), ChartPeriod::Day, Currency::USD), None);
        assert_eq!(price_chart_data(GemChart::mock(vec![]), ChartPeriod::Day, Currency::USD), None);
    }

    #[test]
    fn test_price_chart_data_appends_the_current_point_after_history() {
        let current = GemChartCurrent {
            date: DateTime::from_timestamp(2_000, 0).unwrap(),
            value: 200.0,
            change_percentage: 4.2,
        };
        let chart = GemChart {
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
        assert_eq!(super::super::candlestick_header(100.0, 0.0).change, None);
        assert_eq!(super::super::candlestick_header(100.0, 150.0).change, Some(GemFormattedNumber::percentage(50.0, GemPercentageStyle::Signed)));
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
            chart_sections(&token, Currency::USD, Some(1.0), Some(&market), vec![], links.clone(), Some(BlockExplorerLink::mock())),
            vec![
                set_price_alert(),
                section(
                    GemListSectionTitle::None,
                    vec![
                        GemListRow::Ranked {
                            title: GemListRowTitle::MarketCap,
                            amount: usd(100.0),
                            tag: "#1".to_string()
                        },
                        amount(GemListRowTitle::FullyDilutedValuation, usd(120.0), Some(GemInfoTopic::FullyDilutedValuation)),
                        amount(GemListRowTitle::TradingVolume, usd(10.0), None),
                    ],
                ),
                section(GemListSectionTitle::None, vec![contract(&token, Some(BlockExplorerLink::mock()))]),
                section(
                    GemListSectionTitle::None,
                    vec![
                        amount(GemListRowTitle::CirculatingSupply, supply(50.0, &token.symbol), Some(GemInfoTopic::CirculatingSupply)),
                        amount(GemListRowTitle::TotalSupply, supply(60.0, &token.symbol), Some(GemInfoTopic::TotalSupply)),
                        amount(GemListRowTitle::MaxSupply, supply(21.0, &token.symbol), Some(GemInfoTopic::MaxSupply)),
                    ],
                ),
                section(
                    GemListSectionTitle::None,
                    vec![all_time(GemListRowTitle::AllTimeHigh, ChartValuePercentage::mock()), all_time(GemListRowTitle::AllTimeLow, ChartValuePercentage::mock_low())],
                ),
                section(GemListSectionTitle::SocialLinks, vec![GemListRow::Social { links: social_links(links) }]),
            ]
        );
    }

    #[test]
    fn test_chart_sections_skip_missing_values_and_empty_sections() {
        let sections = chart_sections(&Asset::mock(), Currency::USD, None, Some(&AssetMarket::mock_partial()), vec![], vec![], None);

        assert_eq!(
            sections,
            vec![
                section(GemListSectionTitle::None, vec![amount(GemListRowTitle::FullyDilutedValuation, usd(120.0), Some(GemInfoTopic::FullyDilutedValuation))],),
                section(
                    GemListSectionTitle::None,
                    vec![
                        amount(GemListRowTitle::CirculatingSupply, supply(50.0, &Asset::mock().symbol), Some(GemInfoTopic::CirculatingSupply)),
                        amount(GemListRowTitle::MaxSupply, supply(21.0, &Asset::mock().symbol), Some(GemInfoTopic::MaxSupply)),
                    ],
                ),
                section(GemListSectionTitle::None, vec![all_time(GemListRowTitle::AllTimeHigh, ChartValuePercentage::mock())]),
            ]
        );
    }

    #[test]
    fn test_chart_sections_rank_badge_limit() {
        let rank = |rank: i32| chart_sections(&Asset::mock(), Currency::USD, None, Some(&AssetMarket::mock_with_rank(rank)), vec![], vec![], None).remove(0).rows.remove(0);

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

        assert_eq!(chart_sections(&token, Currency::USD, None, None, vec![], vec![], None), vec![section(GemListSectionTitle::None, vec![contract(&token, None)])]);
    }

    #[test]
    fn test_chart_sections_offer_price_alerts_only_with_a_price() {
        let asset = Asset::mock();
        let auto = PriceAlert::new_auto(asset.id.clone(), Currency::USD);
        let mut notified = PriceAlert::new_price(asset.id.clone(), Currency::USD, 120.0, PriceAlertDirection::Up);
        notified.last_notified_at = Some(Utc::now());
        let first = |price: Option<f64>, alerts: Vec<PriceAlert>| chart_sections(&asset, Currency::USD, price, None, alerts, vec![], None).into_iter().next();

        assert_eq!(first(Some(1.0), vec![]), Some(set_price_alert()));
        assert_eq!(
            first(Some(1.0), vec![auto.clone(), notified.clone()]),
            Some(section(
                GemListSectionTitle::None,
                vec![GemListRow::Link {
                    title: GemListRowTitle::PriceAlerts,
                    value: Some("1".to_string()),
                    icon: GemListRowIcon::None,
                    action: GemRowAction::PriceAlerts,
                }]
            ))
        );
        assert_eq!(first(Some(1.0), vec![notified.clone()]), Some(set_price_alert()));
        assert_eq!(first(Some(0.0), vec![auto.clone()]), None);
        assert_eq!(first(None, vec![auto]), None);
        assert_eq!(first(None, vec![]), None);
    }
}
