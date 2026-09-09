use chrono::{DateTime, Utc};
use primitives::{Asset, AssetMarket, AssetPrice, BlockExplorerLink, ChartDateValue, ChartValue};

use super::model::{GemAssetMarketRow, GemAssetMarketRows};

const MARKET_CAP_RANK_BADGE_LIMIT: i32 = 1000;

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

pub fn current_value(values: &[ChartDateValue], latest: Option<AssetPrice>, now: DateTime<Utc>) -> Option<ChartDateValue> {
    let latest = latest?;
    let is_newer = values.last().is_none_or(|last| latest.updated_at > last.date);
    is_newer.then_some(ChartDateValue { date: now, value: latest.price })
}

pub fn market_rows(asset: &Asset, market: Option<&AssetMarket>, contract_explorer: Option<BlockExplorerLink>) -> GemAssetMarketRows {
    GemAssetMarketRows {
        market: market.map(market_section).unwrap_or_default(),
        contract: available_rows([contract_row(asset, contract_explorer)]),
        supply: market.map(supply_section).unwrap_or_default(),
        all_time: market.map(all_time_section).unwrap_or_default(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, ChartValuePercentage};

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
            price_change_percentage_24h: 0.0,
            updated_at: DateTime::from_timestamp(seconds, 0).unwrap(),
        };
        let now = DateTime::from_timestamp(500, 0).unwrap();

        let current = current_value(&[point(10), point(20)], Some(price(30)), now).expect("current");
        assert_eq!(current.value, 9.0);
        assert_eq!(current.date, now);

        assert_eq!(current_value(&[point(10), point(20)], Some(price(20)), now), None);
        assert_eq!(current_value(&[point(10), point(20)], None, now), None);
        assert!(current_value(&[], Some(price(20)), now).is_some());
    }

    #[test]
    fn test_market_rows() {
        let token = Asset::mock_ethereum_usdc();
        let market = AssetMarket::mock();

        assert_eq!(
            market_rows(&token, Some(&market), Some(BlockExplorerLink::mock())),
            GemAssetMarketRows {
                market: vec![
                    GemAssetMarketRow::MarketCap { value: 100.0, rank: Some(1) },
                    GemAssetMarketRow::FullyDilutedValuation { value: 120.0 },
                    GemAssetMarketRow::TradingVolume { value: 10.0 },
                ],
                contract: vec![GemAssetMarketRow::Contract {
                    token_id: token.id.token_id.clone().unwrap(),
                    explorer: Some(BlockExplorerLink::mock()),
                }],
                supply: vec![
                    GemAssetMarketRow::CirculatingSupply { value: 50.0 },
                    GemAssetMarketRow::TotalSupply { value: 60.0 },
                    GemAssetMarketRow::MaxSupply { value: 21.0 },
                ],
                all_time: vec![
                    GemAssetMarketRow::AllTimeHigh {
                        value: ChartValuePercentage::mock()
                    },
                    GemAssetMarketRow::AllTimeLow {
                        value: ChartValuePercentage::mock_low()
                    },
                ],
            }
        );
    }

    #[test]
    fn test_market_rows_skip_missing_values() {
        let rows = market_rows(&Asset::mock(), Some(&AssetMarket::mock_partial()), None);

        assert_eq!(rows.market, vec![GemAssetMarketRow::FullyDilutedValuation { value: 120.0 }]);
        assert_eq!(rows.contract, Vec::new());
        assert_eq!(
            rows.supply,
            vec![GemAssetMarketRow::CirculatingSupply { value: 50.0 }, GemAssetMarketRow::MaxSupply { value: 21.0 }]
        );
        assert_eq!(
            rows.all_time,
            vec![GemAssetMarketRow::AllTimeHigh {
                value: ChartValuePercentage::mock()
            }]
        );
    }

    #[test]
    fn test_market_rows_rank_badge_limit() {
        let rank = |rank: i32| market_rows(&Asset::mock(), Some(&AssetMarket::mock_with_rank(rank)), None).market[0].clone();

        assert_eq!(rank(MARKET_CAP_RANK_BADGE_LIMIT), GemAssetMarketRow::MarketCap { value: 100.0, rank: Some(1000) });
        assert_eq!(rank(MARKET_CAP_RANK_BADGE_LIMIT + 1), GemAssetMarketRow::MarketCap { value: 100.0, rank: None });
        assert_eq!(rank(0), GemAssetMarketRow::MarketCap { value: 100.0, rank: None });
    }

    #[test]
    fn test_market_rows_without_market_keep_contract() {
        let token = Asset::mock_ethereum_usdc();

        assert_eq!(
            market_rows(&token, None, None),
            GemAssetMarketRows {
                market: Vec::new(),
                contract: vec![GemAssetMarketRow::Contract {
                    token_id: token.id.token_id.clone().unwrap(),
                    explorer: None,
                }],
                supply: Vec::new(),
                all_time: Vec::new(),
            }
        );
    }
}
