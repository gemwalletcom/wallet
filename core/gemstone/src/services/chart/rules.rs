use chrono::{DateTime, Utc};
use primitives::{Asset, AssetLink, AssetMarket, AssetPrice, BlockExplorerLink, ChartDateValue, ChartPeriod, ChartValue, PriceAlert, PriceChangeCalculator};

use super::model::{GemAssetMarketRow, GemChartData, GemChartHeader, GemChartSection, GemChartValueType};
use super::{GemChart, GemChartCurrent};
use crate::services::price::rules::has_price;
use crate::services::price_alert::rules::displayed_price_alert_ids;

const MARKET_CAP_RANK_BADGE_LIMIT: i32 = 1000;
const MIN_CHART_POINTS: usize = 2;

pub fn converted_values(prices: Vec<ChartValue>, rate: f64) -> Vec<ChartDateValue> {
    let mut values: Vec<ChartDateValue> = prices
        .into_iter()
        .filter_map(|price| {
            DateTime::from_timestamp(price.timestamp as i64, 0).map(|date| ChartDateValue {
                date,
                value: price.value as f64 * rate,
            })
        })
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

fn change_percentage(period: ChartPeriod, base_value: f64, latest: &AssetPrice) -> f64 {
    match period {
        ChartPeriod::Day => latest.price_change_percentage_24h,
        ChartPeriod::Hour | ChartPeriod::Week | ChartPeriod::Month | ChartPeriod::Year | ChartPeriod::All => PriceChangeCalculator::percentage(base_value, latest.price),
    }
}

pub fn chart_sections(
    asset: &Asset,
    price: Option<f64>,
    market: Option<&AssetMarket>,
    price_alerts: Vec<PriceAlert>,
    links: Vec<AssetLink>,
    contract_explorer: Option<BlockExplorerLink>,
) -> Vec<GemChartSection> {
    let market_sections = [
        market.map(market_section).unwrap_or_default(),
        available_rows([contract_row(asset, contract_explorer)]),
        market.map(supply_section).unwrap_or_default(),
        market.map(all_time_section).unwrap_or_default(),
    ]
    .into_iter()
    .filter(|rows| !rows.is_empty())
    .map(|rows| GemChartSection::Market { rows });
    price_alert_section(price, price_alerts)
        .into_iter()
        .chain(market_sections)
        .chain((!links.is_empty()).then_some(GemChartSection::Links { links }))
        .collect()
}

fn price_alert_section(price: Option<f64>, price_alerts: Vec<PriceAlert>) -> Option<GemChartSection> {
    if !has_price(price) {
        return None;
    }
    let count = displayed_price_alert_ids(price_alerts).len() as u32;
    Some(if count > 0 { GemChartSection::PriceAlerts { count } } else { GemChartSection::SetPriceAlert })
}

fn market_section(market: &AssetMarket) -> Vec<GemAssetMarketRow> {
    let rank = market.market_cap_rank.filter(|rank| (1..=MARKET_CAP_RANK_BADGE_LIMIT).contains(rank));
    available_rows([
        market.market_cap.map(|value| GemAssetMarketRow::MarketCap { value, rank }),
        market.market_cap_fdv.map(|value| GemAssetMarketRow::FullyDilutedValuation { value }),
        market.total_volume.map(|value| GemAssetMarketRow::TradingVolume { value }),
    ])
}

fn contract_row(asset: &Asset, explorer: Option<BlockExplorerLink>) -> Option<GemAssetMarketRow> {
    let token_id = asset.id.token_id.clone()?;
    Some(GemAssetMarketRow::Contract { token_id, explorer })
}

fn supply_section(market: &AssetMarket) -> Vec<GemAssetMarketRow> {
    available_rows([
        market.circulating_supply.map(|value| GemAssetMarketRow::CirculatingSupply { value }),
        market.total_supply.map(|value| GemAssetMarketRow::TotalSupply { value }),
        market.max_supply.map(|value| GemAssetMarketRow::MaxSupply { value }),
    ])
}

fn all_time_section(market: &AssetMarket) -> Vec<GemAssetMarketRow> {
    available_rows([
        market.all_time_high_value.clone().map(|value| GemAssetMarketRow::AllTimeHigh { value }),
        market.all_time_low_value.clone().map(|value| GemAssetMarketRow::AllTimeLow { value }),
    ])
}

fn available_rows<const N: usize>(rows: [Option<GemAssetMarketRow>; N]) -> Vec<GemAssetMarketRow> {
    rows.into_iter().flatten().collect()
}

pub fn price_chart_data(chart: GemChart) -> Option<GemChartData> {
    let base = chart.base_value;
    let current = chart.current;
    let values: Vec<ChartDateValue> = chart
        .values
        .into_iter()
        .chain(current.as_ref().map(|current| ChartDateValue {
            date: current.date,
            value: current.value,
        }))
        .collect();
    if values.len() < MIN_CHART_POINTS {
        return None;
    }
    let last = values.last()?.value;
    let header = match current {
        Some(current) => header(GemChartValueType::Price, base, current.value, Some(current.change_percentage), false),
        None => header(GemChartValueType::Price, base, last, None, false),
    };
    Some(GemChartData {
        value_type: GemChartValueType::Price,
        base,
        shows_secondary_value: false,
        values,
        header: Some(header),
    })
}

pub fn change_chart_data(values: Vec<ChartDateValue>, shows_secondary_value: bool) -> Option<GemChartData> {
    if values.len() < MIN_CHART_POINTS || !has_variation(&values) {
        return None;
    }
    let base = values.first()?.value;
    let last = values.last()?.value;
    Some(GemChartData {
        value_type: GemChartValueType::PriceChange,
        base,
        shows_secondary_value,
        values,
        header: Some(header(GemChartValueType::PriceChange, base, last, None, shows_secondary_value)),
    })
}

pub fn header(value_type: GemChartValueType, base: f64, value: f64, change_percentage: Option<f64>, shows_secondary_value: bool) -> GemChartHeader {
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
    GemChartHeader {
        value: display_value,
        secondary_value,
        change_percentage: shows_change.then_some(change_percentage),
    }
}

fn has_variation(values: &[ChartDateValue]) -> bool {
    let first = values.first().map(|value| value.value);
    values.iter().any(|value| Some(value.value) != first)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, ChartValuePercentage, LinkType, PriceAlertDirection, currency::Currency};

    fn point(seconds: i64, value: f64) -> ChartDateValue {
        ChartDateValue {
            date: DateTime::from_timestamp(seconds, 0).unwrap(),
            value,
        }
    }

    fn chart(values: Vec<ChartDateValue>, current: Option<GemChartCurrent>) -> GemChart {
        let base_value = base_value(&values);
        GemChart { values, base_value, current }
    }

    #[test]
    fn test_price_chart_data_needs_two_points() {
        assert_eq!(price_chart_data(chart(vec![point(1, 100.0)], None)), None);
        assert_eq!(price_chart_data(chart(vec![], None)), None);
    }

    #[test]
    fn test_price_chart_data_appends_the_current_point_after_history() {
        let current = GemChartCurrent {
            date: DateTime::from_timestamp(2_000, 0).unwrap(),
            value: 200.0,
            change_percentage: 4.2,
        };
        let data = price_chart_data(chart(vec![point(1_000, 100.0)], Some(current))).expect("data");

        assert_eq!(data.value_type, GemChartValueType::Price);
        assert_eq!(data.base, 100.0);
        assert_eq!(data.values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![100.0, 200.0]);
        assert_eq!(
            data.header,
            Some(GemChartHeader {
                value: 200.0,
                secondary_value: None,
                change_percentage: Some(4.2),
            })
        );
    }

    #[test]
    fn test_price_chart_data_header_falls_back_to_the_last_point() {
        let data = price_chart_data(chart(vec![point(1, 100.0), point(2, 150.0)], None)).expect("data");

        assert_eq!(
            data.header,
            Some(GemChartHeader {
                value: 150.0,
                secondary_value: None,
                change_percentage: Some(50.0),
            })
        );
    }

    #[test]
    fn test_change_chart_data_needs_a_series_that_moves() {
        assert_eq!(change_chart_data(vec![point(1, 5.0), point(2, 5.0), point(3, 5.0)], true), None);
        assert_eq!(change_chart_data(vec![point(1, 5.0)], true), None);
    }

    #[test]
    fn test_change_chart_data_header_is_the_distance_from_the_first_value() {
        let data = change_chart_data(vec![point(1, 10.0), point(2, 12.0), point(3, 15.0)], true).expect("data");

        assert_eq!(data.value_type, GemChartValueType::PriceChange);
        assert_eq!(data.base, 10.0);
        assert_eq!(
            data.header,
            Some(GemChartHeader {
                value: 5.0,
                secondary_value: Some(15.0),
                change_percentage: Some(50.0),
            })
        );
        assert_eq!(header(data.value_type, data.base, 12.0, None, true).value, 2.0);
    }

    #[test]
    fn test_change_chart_data_hides_the_percentage_without_a_secondary_value() {
        let values = vec![point(1, 10.0), point(2, 12.0)];

        assert_eq!(change_chart_data(values.clone(), false).unwrap().header.unwrap().change_percentage, None);
        assert_eq!(change_chart_data(values, true).unwrap().header.unwrap().change_percentage, Some(20.0));
    }

    #[test]
    fn test_header_hides_the_percentage_of_a_zero_value() {
        assert_eq!(header(GemChartValueType::Price, 100.0, 0.0, None, false).change_percentage, None);
        assert_eq!(header(GemChartValueType::Price, 100.0, 150.0, None, false).change_percentage, Some(50.0));
    }

    #[test]
    fn test_converted_values_apply_rate_and_sort() {
        let values = converted_values(vec![ChartValue { timestamp: 20, value: 2.0 }, ChartValue { timestamp: 10, value: 1.5 }], 2.0);
        assert_eq!(values.iter().map(|value| value.date.timestamp()).collect::<Vec<_>>(), vec![10, 20]);
        assert_eq!(values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![3.0, 4.0]);
    }

    #[test]
    fn test_current_value_is_only_a_price_newer_than_the_chart() {
        let point = |seconds: i64| ChartDateValue {
            date: DateTime::from_timestamp(seconds, 0).unwrap(),
            value: 1.0,
        };
        let price = |seconds: i64| AssetPrice {
            asset_id: AssetId::from_chain(primitives::Chain::Bitcoin),
            price: 9.0,
            price_change_percentage_24h: 4.2,
            updated_at: DateTime::from_timestamp(seconds, 0).unwrap(),
        };
        let now = DateTime::from_timestamp(500, 0).unwrap();

        let current = current_value(&[point(10), point(20)], Some(price(30)), now, ChartPeriod::Day, 1.0).expect("current");
        assert_eq!(current.value, 9.0);
        assert_eq!(current.date, now);
        assert_eq!(current.change_percentage, 4.2);
        assert_eq!(
            current_value(&[point(10), point(20)], Some(price(30)), now, ChartPeriod::Week, 3.0)
                .unwrap()
                .change_percentage,
            200.0
        );
        assert_eq!(
            current_value(&[point(10), point(20)], Some(price(30)), now, ChartPeriod::Week, 0.0)
                .unwrap()
                .change_percentage,
            0.0
        );

        assert_eq!(current_value(&[point(10), point(20)], Some(price(20)), now, ChartPeriod::Day, 1.0), None);
        assert_eq!(current_value(&[point(10), point(20)], None, now, ChartPeriod::Day, 1.0), None);
        assert!(current_value(&[], Some(price(20)), now, ChartPeriod::Day, 0.0).is_some());
    }

    #[test]
    fn test_base_value_is_the_first_non_zero_value() {
        let point = |value: f64| ChartDateValue {
            date: DateTime::from_timestamp(0, 0).unwrap(),
            value,
        };
        assert_eq!(base_value(&[point(0.0), point(100.0), point(200.0)]), 100.0);
        assert_eq!(base_value(&[point(50.0), point(100.0)]), 50.0);
        assert_eq!(base_value(&[point(0.0)]), 0.0);
        assert_eq!(base_value(&[]), 0.0);
    }

    #[test]
    fn test_chart_sections_group_the_market_rows_core_has() {
        let token = Asset::mock_ethereum_usdc();
        let market = AssetMarket::mock();
        let links = vec![AssetLink::new("https://example.com", LinkType::Website)];

        assert_eq!(
            chart_sections(&token, Some(1.0), Some(&market), vec![], links.clone(), Some(BlockExplorerLink::mock())),
            vec![
                GemChartSection::SetPriceAlert,
                GemChartSection::Market {
                    rows: vec![
                        GemAssetMarketRow::MarketCap { value: 100.0, rank: Some(1) },
                        GemAssetMarketRow::FullyDilutedValuation { value: 120.0 },
                        GemAssetMarketRow::TradingVolume { value: 10.0 },
                    ]
                },
                GemChartSection::Market {
                    rows: vec![GemAssetMarketRow::Contract {
                        token_id: token.id.token_id.clone().unwrap(),
                        explorer: Some(BlockExplorerLink::mock()),
                    }]
                },
                GemChartSection::Market {
                    rows: vec![
                        GemAssetMarketRow::CirculatingSupply { value: 50.0 },
                        GemAssetMarketRow::TotalSupply { value: 60.0 },
                        GemAssetMarketRow::MaxSupply { value: 21.0 },
                    ]
                },
                GemChartSection::Market {
                    rows: vec![
                        GemAssetMarketRow::AllTimeHigh {
                            value: ChartValuePercentage::mock()
                        },
                        GemAssetMarketRow::AllTimeLow {
                            value: ChartValuePercentage::mock_low()
                        },
                    ]
                },
                GemChartSection::Links { links },
            ]
        );
    }

    #[test]
    fn test_chart_sections_skip_missing_values_and_empty_sections() {
        let sections = chart_sections(&Asset::mock(), None, Some(&AssetMarket::mock_partial()), vec![], vec![], None);

        assert_eq!(
            sections,
            vec![
                GemChartSection::Market {
                    rows: vec![GemAssetMarketRow::FullyDilutedValuation { value: 120.0 }]
                },
                GemChartSection::Market {
                    rows: vec![GemAssetMarketRow::CirculatingSupply { value: 50.0 }, GemAssetMarketRow::MaxSupply { value: 21.0 }]
                },
                GemChartSection::Market {
                    rows: vec![GemAssetMarketRow::AllTimeHigh {
                        value: ChartValuePercentage::mock()
                    }]
                },
            ]
        );
    }

    #[test]
    fn test_chart_sections_rank_badge_limit() {
        let rank = |rank: i32| match chart_sections(&Asset::mock(), None, Some(&AssetMarket::mock_with_rank(rank)), vec![], vec![], None).remove(0) {
            GemChartSection::Market { rows } => rows[0].clone(),
            section => panic!("expected market rows, got {section:?}"),
        };

        assert_eq!(rank(MARKET_CAP_RANK_BADGE_LIMIT), GemAssetMarketRow::MarketCap { value: 100.0, rank: Some(1000) });
        assert_eq!(rank(MARKET_CAP_RANK_BADGE_LIMIT + 1), GemAssetMarketRow::MarketCap { value: 100.0, rank: None });
        assert_eq!(rank(0), GemAssetMarketRow::MarketCap { value: 100.0, rank: None });
    }

    #[test]
    fn test_chart_sections_without_market_keep_contract() {
        let token = Asset::mock_ethereum_usdc();

        assert_eq!(
            chart_sections(&token, None, None, vec![], vec![], None),
            vec![GemChartSection::Market {
                rows: vec![GemAssetMarketRow::Contract {
                    token_id: token.id.token_id.clone().unwrap(),
                    explorer: None,
                }]
            }]
        );
    }

    #[test]
    fn test_chart_sections_offer_price_alerts_only_with_a_price() {
        let asset = Asset::mock();
        let auto = PriceAlert::new_auto(asset.id.clone(), Currency::USD);
        let mut notified = PriceAlert::new_price(asset.id.clone(), Currency::USD, 120.0, PriceAlertDirection::Up);
        notified.last_notified_at = Some(Utc::now());
        let section = |price: Option<f64>, alerts: Vec<PriceAlert>| chart_sections(&asset, price, None, alerts, vec![], None).into_iter().next();

        assert_eq!(section(Some(1.0), vec![]), Some(GemChartSection::SetPriceAlert));
        assert_eq!(section(Some(1.0), vec![auto.clone(), notified.clone()]), Some(GemChartSection::PriceAlerts { count: 1 }));
        assert_eq!(section(Some(1.0), vec![notified.clone()]), Some(GemChartSection::SetPriceAlert));
        assert_eq!(section(Some(0.0), vec![auto.clone()]), None);
        assert_eq!(section(None, vec![auto]), None);
        assert_eq!(section(None, vec![]), None);
    }
}
