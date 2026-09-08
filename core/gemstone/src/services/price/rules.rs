use std::collections::HashMap;

use crate::services::collections::unique;
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

pub fn changed_rate(stored: Option<FiatRate>, rates: &[FiatRate], currency: &Currency) -> Option<f64> {
    let rate = rates.iter().find(|rate| rate.symbol == *currency)?.rate;
    (stored.map(|stored| stored.rate) != Some(rate)).then_some(rate)
}

pub fn changed_prices(stored: Vec<AssetPrice>, updates: Vec<GemPriceUpdate>) -> Vec<GemPriceUpdate> {
    let stored: HashMap<AssetId, AssetPrice> = stored.into_iter().map(|price| (price.asset_id.clone(), price)).collect();
    updates
        .into_iter()
        .filter(|update| {
            stored
                .get(&update.asset_id)
                .is_none_or(|price| price.price != update.price || price.price_change_percentage_24h != update.price_change_percentage_24h)
        })
        .collect()
}

pub fn observable_asset_ids(enabled: Vec<AssetId>, alerts: Vec<AssetId>, defaults: Vec<AssetId>) -> Vec<AssetId> {
    let asset_ids = unique(enabled.into_iter().chain(alerts));
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

    #[test]
    fn test_rate_or_base_only_defaults_usd() {
        let stored = FiatRate { symbol: Currency::EUR, rate: 0.9 };
        assert_eq!(rate_or_base(Currency::EUR, Some(stored.clone())).map(|rate| rate.rate), Some(0.9));
        assert_eq!(rate_or_base(Currency::USD, None).map(|rate| rate.rate), Some(1.0));
        assert!(rate_or_base(Currency::EUR, None).is_none());
    }

    #[test]
    fn test_changed_rate_only_reports_a_moved_rate_for_the_current_currency() {
        let stored = FiatRate { symbol: Currency::EUR, rate: 0.9 };
        let rates = |rate| vec![FiatRate { symbol: Currency::EUR, rate }, FiatRate { symbol: Currency::GBP, rate: 0.8 }];

        assert_eq!(changed_rate(Some(stored.clone()), &rates(0.9), &Currency::EUR), None);
        assert_eq!(changed_rate(Some(stored.clone()), &rates(1.1), &Currency::EUR), Some(1.1));
        assert_eq!(changed_rate(None, &rates(0.9), &Currency::EUR), Some(0.9));
        assert_eq!(changed_rate(Some(stored), &rates(0.9), &Currency::JPY), None);
    }

    #[test]
    fn test_changed_prices_keeps_only_moved_values() {
        let now = chrono::Utc::now();
        let bitcoin = AssetId::from_chain(primitives::Chain::Bitcoin);
        let ethereum = AssetId::from_chain(primitives::Chain::Ethereum);
        let stored = vec![AssetPrice::new(bitcoin.clone(), 100.0, 2.0, now), AssetPrice::new(ethereum.clone(), 50.0, 1.0, now)];
        let update = |asset_id: &AssetId, price, change| GemPriceUpdate {
            asset_id: asset_id.clone(),
            price,
            price_usd: price,
            price_change_percentage_24h: change,
            updated_at: now + chrono::Duration::seconds(60),
        };

        let changed = changed_prices(stored.clone(), vec![update(&bitcoin, 100.0, 2.0), update(&ethereum, 50.0, 1.0)]);
        assert!(changed.is_empty());

        let changed = changed_prices(stored.clone(), vec![update(&bitcoin, 100.0, 2.5), update(&ethereum, 51.0, 1.0)]);
        assert_eq!(changed.iter().map(|update| update.asset_id.clone()).collect::<Vec<_>>(), vec![bitcoin, ethereum]);

        let solana = AssetId::from_chain(primitives::Chain::Solana);
        let changed = changed_prices(stored, vec![update(&solana, 10.0, 0.0)]);
        assert_eq!(changed.len(), 1);
    }

    #[test]
    fn test_fiat_prices_convert_with_rate() {
        let now = chrono::Utc::now();
        let rate = FiatRate { symbol: Currency::EUR, rate: 0.5 };

        let updates = fiat_prices(vec![AssetPrice::new(AssetId::from_chain(primitives::Chain::Bitcoin), 100.0, 2.0, now)], &rate);

        assert_eq!((updates[0].price, updates[0].price_usd, updates[0].price_change_percentage_24h), (50.0, 100.0, 2.0));
        assert_eq!(updates[0].updated_at, now);
    }
}
