use primitives::currency::Currency;
use primitives::{AssetPrice, FiatRate};

use super::model::GemPriceUpdate;

pub fn rate_or_base(currency: Currency, stored: Option<FiatRate>) -> Option<FiatRate> {
    stored.or_else(|| (currency == Currency::USD).then_some(FiatRate { symbol: Currency::USD, rate: 1.0 }))
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
