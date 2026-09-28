use crate::ConfigCacher;
use crate::prices::PriceClient;
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;

use config_keys::ConfigKey;
use storage::{Database, DatabaseError, PricesRepository};
use streamer::{PricesPayload, consumer::MessageConsumer};

pub struct StorePricesConsumer {
    pub database: Database,
    pub price_client: PriceClient,
    pub config: Arc<ConfigCacher>,
}

impl StorePricesConsumer {
    pub fn new(database: Database, price_client: PriceClient, config: Arc<ConfigCacher>) -> Self {
        Self { database, price_client, config }
    }
}

#[async_trait]
impl MessageConsumer<PricesPayload, usize> for StorePricesConsumer {
    async fn should_process(&self, _payload: &PricesPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: PricesPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let prices = payload.prices;
        let primary_price_max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        let ttl_seconds = self.config.get_duration(ConfigKey::PriceOutdated).await?.as_secs() as i64;
        let (count, cache_entries) = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let asset_ids = client.set_prices(prices)?;
                if asset_ids.is_empty() {
                    return Ok((0, Vec::new()));
                }

                let cache_entries = client.get_primary_price_infos(&asset_ids, primary_price_max_age)?;
                Ok((cache_entries.len(), cache_entries))
            })
            .await?;
        if count == 0 {
            return Ok(0);
        }
        self.price_client.set_cache_prices(cache_entries, ttl_seconds).await?;

        Ok(count)
    }
}
