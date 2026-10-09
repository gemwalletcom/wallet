use std::error::Error;
use std::sync::Arc;

use config_keys::ConfigKey;
use primitives::{AssetId, ChartPeriod, ChartValue, currency::Currency};

use super::repository::{ChartData, Repository};

use crate::ConfigCacher;

#[derive(Clone)]
pub struct ChartClient {
    repository: Arc<dyn Repository>,
    config: Arc<ConfigCacher>,
}

impl ChartClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, config: Arc<ConfigCacher>) -> Self {
        Self { repository, config }
    }

    pub async fn get_charts_prices(&self, asset_id: &AssetId, period: ChartPeriod, currency: &Currency) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let primary_price_max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        let ChartData { base_rate, rate, charts } = self.repository.get_chart_data(asset_id.clone(), currency.clone(), period, primary_price_max_age).await?;
        let rate_multiplier = rate.multiplier(base_rate.rate);
        Ok(charts
            .into_iter()
            .map(|(ts, price)| ChartValue {
                timestamp: ts.and_utc().timestamp() as i32,
                value: (price * rate_multiplier) as f32,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use primitives::{AssetId, Chain, FiatRate};

    use super::*;
    use crate::testkit::{MemoryConfigRepository, MemoryPricesRepository};

    #[tokio::test]
    async fn test_get_charts_prices_converts_to_currency() {
        let at = DateTime::from_timestamp(1_700_000_000, 0).unwrap().naive_utc();
        let repository = MemoryPricesRepository::default()
            .with_fiat_rates(vec![FiatRate { symbol: Currency::USD, rate: 1.0 }, FiatRate { symbol: Currency::EUR, rate: 0.5 }])
            .with_charts(vec![(at, 100.0)]);
        let config = Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new())));
        let client = ChartClient::new(Arc::new(repository), config);

        let values = client.get_charts_prices(&AssetId::from_chain(Chain::Bitcoin), ChartPeriod::Day, &Currency::EUR).await.unwrap();

        assert_eq!(values, vec![ChartValue { timestamp: 1_700_000_000, value: 50.0 }]);
    }
}
