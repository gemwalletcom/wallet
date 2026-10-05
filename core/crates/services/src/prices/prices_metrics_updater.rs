use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use chrono::{Duration, Utc};
use primitives::{PriceData, PriceProvider};

use super::repository::Repository;

pub struct PricesMetricsUpdater {
    repository: Arc<dyn Repository>,
    provider: PriceProvider,
}

impl PricesMetricsUpdater {
    pub(crate) fn new(repository: Arc<dyn Repository>, provider: PriceProvider) -> Self {
        Self { repository, provider }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if self.provider.supports_price_change_24h() {
            return Ok(0);
        }
        let now = Utc::now();
        let from = (now - Duration::hours(25)).naive_utc();
        let until = (now - Duration::hours(24)).naive_utc();
        Ok(self.repository.update_price_changes(self.provider, from, until).await?)
    }
}

pub(crate) fn price_changes(prices: &[PriceData], previous: &HashMap<String, f64>) -> Vec<(String, f64)> {
    prices
        .iter()
        .filter(|price| price.price != 0.0)
        .filter_map(|price| {
            let price_id = price.id.to_string();
            let prev = previous.get(&price_id).copied().unwrap_or(0.0);
            (prev != 0.0).then(|| (price_id, (price.price - prev) / prev * 100.0))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use primitives::PriceId;

    use super::*;

    fn price(id: &str, price: f64) -> PriceData {
        PriceData {
            id: PriceId::new(PriceProvider::Coingecko, id.to_string()),
            price,
            ..PriceData::mock()
        }
    }

    #[test]
    fn test_price_changes() {
        let prices = [price("up", 110.0), price("zero", 0.0), price("new", 50.0), price("flat", 10.0)];
        let previous = HashMap::from([("coingecko_up".to_string(), 100.0), ("coingecko_zero".to_string(), 5.0), ("coingecko_flat".to_string(), 0.0)]);

        let changes = price_changes(&prices, &previous);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].0, "coingecko_up");
        assert!((changes[0].1 - 10.0).abs() < 1e-9);
    }
}
