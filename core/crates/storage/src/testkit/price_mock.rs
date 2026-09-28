use crate::models::PriceRow;
use crate::sql_types::{PriceId, PriceProviderRow};
use chrono::{Duration, NaiveDateTime, Utc};
use primitives::{PriceId as PrimitivePriceId, PriceProvider};

impl PriceRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider: PriceProvider,
        provider_price_id: String,
        price: f64,
        price_change_percentage_24h: Option<f64>,
        all_time_high: f64,
        all_time_high_date: Option<NaiveDateTime>,
        all_time_low: f64,
        all_time_low_date: Option<NaiveDateTime>,
        market_cap_rank: Option<i32>,
        total_volume: Option<f64>,
        last_updated_at: NaiveDateTime,
    ) -> Self {
        let id = PrimitivePriceId::new(provider, provider_price_id);
        PriceRow {
            id: id.into(),
            provider: provider.into(),
            price,
            price_change_percentage_24h,
            last_updated_at,
            all_time_high,
            all_time_high_date,
            all_time_low,
            all_time_low_date,
            market_cap_rank,
            total_volume,
        }
    }

    pub fn mock_with_age(provider: PriceProvider, seconds_ago: i64) -> Self {
        Self {
            id: PriceId::from(PrimitivePriceId::new(provider, "bitcoin".to_string())),
            provider: PriceProviderRow(provider),
            price: 1.0,
            price_change_percentage_24h: None,
            all_time_high: 0.0,
            all_time_high_date: None,
            all_time_low: 0.0,
            all_time_low_date: None,
            market_cap_rank: None,
            total_volume: None,
            last_updated_at: (Utc::now() - Duration::seconds(seconds_ago)).naive_utc(),
        }
    }
}
