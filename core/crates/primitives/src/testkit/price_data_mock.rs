use chrono::Utc;

use crate::{PriceData, PriceId, PriceProvider};

impl PriceData {
    pub fn mock() -> Self {
        Self::mock_with(80_000.0, 1.5)
    }

    pub fn mock_with(price: f64, price_change_percentage_24h: f64) -> Self {
        Self {
            id: PriceId::new(PriceProvider::Coingecko, "bitcoin".to_string()),
            provider: PriceProvider::Coingecko,
            provider_price_id: "bitcoin".to_string(),
            price,
            price_change_percentage_24h,
            all_time_high: 126_080.0,
            all_time_high_date: None,
            all_time_low: 67.81,
            all_time_low_date: None,
            market_cap_rank: Some(1),
            total_volume: None,
            last_updated_at: Utc::now(),
        }
    }
}
