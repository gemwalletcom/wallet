use primitives::currency::Currency;
use primitives::{AssetId, AssetMarket, AssetPrice, FiatRate};

use super::model::GemPriceUpdate;

pub fn rate_or_base(currency: Currency, stored: Option<FiatRate>) -> Option<FiatRate> {
    stored.or_else(|| (currency == Currency::USD).then_some(FiatRate { symbol: Currency::USD, rate: 1.0 }))
}

pub fn market_in_currency(market: AssetMarket, rate: f64) -> AssetMarket {
    let convert = |value: Option<f64>| value.map(|value| value * rate);
    AssetMarket {
        market_cap: convert(market.market_cap),
        market_cap_fdv: convert(market.market_cap_fdv),
        total_volume: convert(market.total_volume),
        all_time_high_value: market.all_time_high_value.map(|value| value.with_rate(rate)),
        all_time_low_value: market.all_time_low_value.map(|value| value.with_rate(rate)),
        ..market
    }
}

pub fn fiat_prices(prices: Vec<AssetPrice>, rate: &FiatRate) -> Vec<GemPriceUpdate> {
    prices
        .into_iter()
        .map(|price| GemPriceUpdate {
            asset_id: price.asset_id,
            price: rate.multiplier(price.price),
            price_usd: price.price,
            price_change_percentage_24h: price.price_change_percentage_24h,
            updated_at: price.updated_at,
        })
        .collect()
}

pub fn observable_asset_ids(enabled: Vec<AssetId>, alerts: Vec<AssetId>, defaults: Vec<AssetId>) -> Vec<AssetId> {
    let mut seen = std::collections::HashSet::new();
    let asset_ids: Vec<AssetId> = enabled.into_iter().chain(alerts).filter(|asset_id| seen.insert(asset_id.clone())).collect();
    if asset_ids.is_empty() { defaults } else { asset_ids }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use primitives::ChartValuePercentage;

    #[test]
    fn test_market_in_currency_scales_fiat_figures_only() {
        let market = AssetMarket {
            market_cap: Some(1_000.0),
            market_cap_fdv: Some(1_500.0),
            market_cap_rank: Some(1),
            total_volume: Some(200.0),
            circulating_supply: Some(10.0),
            total_supply: Some(20.0),
            max_supply: Some(21.0),
            all_time_high_value: Some(ChartValuePercentage {
                date: Utc::now(),
                value: 300.0,
                percentage: -10.0,
            }),
            ..Default::default()
        };

        let converted = market_in_currency(market, 0.5);

        assert_eq!(converted.market_cap, Some(500.0));
        assert_eq!(converted.market_cap_fdv, Some(750.0));
        assert_eq!(converted.total_volume, Some(100.0));
        assert_eq!(converted.all_time_high_value.as_ref().map(|value| value.value), Some(150.0));
        assert_eq!(converted.all_time_high_value.as_ref().map(|value| value.percentage), Some(-10.0));
        assert_eq!(converted.circulating_supply, Some(10.0));
        assert_eq!(converted.market_cap_rank, Some(1));
    }
}

#[cfg(test)]
mod observable_tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_observable_asset_ids_falls_back_to_defaults() {
        let bitcoin = AssetId::from_chain(Chain::Bitcoin);
        let ethereum = AssetId::from_chain(Chain::Ethereum);
        assert_eq!(observable_asset_ids(vec![bitcoin.clone()], vec![], vec![ethereum.clone()]), vec![bitcoin.clone()]);
        assert_eq!(observable_asset_ids(vec![], vec![], vec![ethereum.clone()]), vec![ethereum.clone()]);
        assert_eq!(
            observable_asset_ids(vec![bitcoin.clone()], vec![bitcoin.clone(), ethereum.clone()], vec![]),
            vec![bitcoin, ethereum]
        );
    }
}
