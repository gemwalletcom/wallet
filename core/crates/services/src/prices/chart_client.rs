use std::error::Error;
use std::sync::Arc;

use config_keys::ConfigKey;
use primitives::{AssetId, ChartPeriod, ChartValue, currency::Currency};
use storage::{ChartsRepository, Database, FiatRepository, PricesRepository};

use crate::ConfigCacher;

#[derive(Clone)]
pub struct ChartClient {
    database: Database,
    config: Arc<ConfigCacher>,
}

impl ChartClient {
    pub fn new(database: Database, config: Arc<ConfigCacher>) -> Self {
        Self { database, config }
    }

    pub async fn get_charts_prices(&self, asset_id: &AssetId, period: ChartPeriod, currency: &Currency) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let asset_id = asset_id.clone();
        let currency = currency.clone();
        let primary_price_max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        let (rate_multiplier, charts) = self
            .database
            .run(move |client| -> Result<_, Box<dyn Error + Send + Sync>> {
                let base_rate = client.get_fiat_rate(&Currency::USD)?;
                let rate = client.get_fiat_rate(&currency)?;
                let key = client.get_primary_price_key(&asset_id, primary_price_max_age)?;
                Ok((rate.multiplier(base_rate.rate), client.get_charts(&key.id(), &period)?))
            })
            .await?;
        Ok(charts
            .into_iter()
            .map(|(ts, price)| ChartValue {
                timestamp: ts.and_utc().timestamp() as i32,
                value: (price * rate_multiplier) as f32,
            })
            .collect())
    }
}
