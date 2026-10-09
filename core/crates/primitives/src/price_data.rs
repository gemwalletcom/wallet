use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Price, PriceChangeCalculator, PriceId, PriceProvider};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub id: PriceId,
    pub provider: PriceProvider,
    pub provider_price_id: String,
    pub price: f64,
    pub price_change_percentage_24h: Option<f64>,
    pub all_time_high: f64,
    pub all_time_high_date: Option<DateTime<Utc>>,
    pub all_time_low: f64,
    pub all_time_low_date: Option<DateTime<Utc>>,
    pub market_cap: Option<f64>,
    pub market_cap_fdv: Option<f64>,
    pub market_cap_rank: Option<i32>,
    pub total_volume: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub last_updated_at: DateTime<Utc>,
}

impl PriceData {
    pub fn as_price(&self) -> Price {
        Price::new(self.price, self.price_change_percentage_24h.unwrap_or_default(), self.last_updated_at, self.provider)
    }

    pub fn with_price_change_from(self, price_24h_ago: Option<f64>) -> Self {
        let price_change_percentage_24h = self
            .price_change_percentage_24h
            .or_else(|| price_24h_ago.filter(|previous| *previous != 0.0 && self.price != 0.0).map(|previous| PriceChangeCalculator::percentage(previous, self.price)));
        Self { price_change_percentage_24h, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_price_change_from() {
        let unknown = PriceData {
            price_change_percentage_24h: None,
            ..PriceData::mock_with(110.0, 0.0)
        };

        assert!((unknown.clone().with_price_change_from(Some(100.0)).price_change_percentage_24h.unwrap() - 10.0).abs() < 1e-9);
        assert_eq!(unknown.clone().with_price_change_from(None).price_change_percentage_24h, None);
        assert_eq!(unknown.clone().with_price_change_from(Some(0.0)).price_change_percentage_24h, None);
        assert_eq!(PriceData { price: 0.0, ..unknown }.with_price_change_from(Some(100.0)).price_change_percentage_24h, None);
        assert_eq!(PriceData::mock_with(110.0, -2.5).with_price_change_from(Some(100.0)).price_change_percentage_24h, Some(-2.5));
    }
}
