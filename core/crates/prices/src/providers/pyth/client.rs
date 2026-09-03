use std::error::Error;

use super::model::{HermesResponse, Price, PriceFeed};
use super::target::PythTarget;
use gem_client::{Client, ClientExt};

const PRICE_IDS_PER_REQUEST: usize = 5;

pub struct PythClient<C: Client> {
    client: C,
}

impl<C: Client> PythClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_price_feeds(&self) -> Result<Vec<PriceFeed>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(PythTarget::PriceFeeds).await?)
    }

    pub async fn get_asset_prices(&self, price_ids: Vec<String>) -> Result<Vec<Price>, Box<dyn Error + Send + Sync>> {
        let mut all_prices = Vec::new();

        for chunk in price_ids.chunks(PRICE_IDS_PER_REQUEST) {
            let response: HermesResponse = self.client.get(PythTarget::LatestPrices { ids: chunk.to_vec() }).await?;

            let prices: Vec<Price> = response
                .parsed
                .into_iter()
                .map(|feed| {
                    let scaled_price = feed.price.price as f64 * 10f64.powi(feed.price.expo);
                    Price { price: scaled_price }
                })
                .collect();

            all_prices.extend(prices);
        }

        Ok(all_prices)
    }
}
