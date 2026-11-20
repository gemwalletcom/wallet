use crate::models::PriceRow;
use crate::sql_types::{PriceId, PriceProviderRow};
use primitives::{DateTime, PriceId as PrimitivePriceId, PriceProvider};
use std::time::Duration;

impl PriceRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider: PriceProvider,
        provider_price_id: String,
        price: f64,
        price_change_percentage_24h: Option<f64>,
        all_time_high: f64,
        all_time_high_date: Option<DateTime>,
        all_time_low: f64,
        all_time_low_date: Option<DateTime>,
        market_cap_rank: Option<i32>,
        total_volume: Option<f64>,
        last_updated_at: DateTime,
    ) -> Self {
        let id = PrimitivePriceId::new(provider, provider_price_id);
        PriceRow {
            id: id.into(),
            provider: provider.into(),
            price,
            price_change_percentage_24h,
            last_updated_at: last_updated_at.into(),
            all_time_high,
            all_time_high_date: all_time_high_date.map(Into::into),
            all_time_low,
            all_time_low_date: all_time_low_date.map(Into::into),
            market_cap: None,
            market_cap_fdv: None,
            market_cap_rank,
            total_volume,
            circulating_supply: None,
            total_supply: None,
            max_supply: None,
        }
    }

    pub fn mock_with_age(provider: PriceProvider, seconds_ago: u64) -> Self {
        Self {
            id: PriceId::from(PrimitivePriceId::new(provider, "bitcoin".to_string())),
            provider: PriceProviderRow(provider),
            price: 1.0,
            price_change_percentage_24h: None,
            all_time_high: 0.0,
            all_time_high_date: None,
            all_time_low: 0.0,
            all_time_low_date: None,
            market_cap: None,
            market_cap_fdv: None,
            market_cap_rank: None,
            total_volume: None,
            circulating_supply: None,
            total_supply: None,
            max_supply: None,
            last_updated_at: (DateTime::now() - Duration::from_secs(seconds_ago)).into(),
        }
    }
}
