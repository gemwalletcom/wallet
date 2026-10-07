use crate::ConfigCacher;
use crate::prices::PriceClient;
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;

use config_keys::ConfigKey;
use streamer::{PricesPayload, consumer::MessageConsumer};

pub struct StorePricesConsumer {
    pub price_client: PriceClient,
    pub config: Arc<ConfigCacher>,
}

impl StorePricesConsumer {
    pub(crate) fn new(price_client: PriceClient, config: Arc<ConfigCacher>) -> Self {
        Self { price_client, config }
    }
}

#[async_trait]
impl MessageConsumer<PricesPayload, usize> for StorePricesConsumer {
    async fn should_consume(&self, _payload: &PricesPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn consume(&self, payload: PricesPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let prices = payload.prices;
        let primary_price_max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        let ttl = self.config.get_duration(ConfigKey::PriceOutdated).await?;
        let cache_entries = self.price_client.store_prices(prices, primary_price_max_age).await?;
        let count = cache_entries.len();
        if count == 0 {
            return Ok(0);
        }
        self.price_client.set_cache_prices(cache_entries, ttl).await?;

        Ok(count)
    }
}
