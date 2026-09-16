use std::error::Error;

use async_trait::async_trait;
use coingecko::{CoinGeckoClient, MAX_MARKETS_PER_PAGE};

use super::mapper;
use crate::config::CoingeckoConfig;
use crate::providers::{ImageListProvider, ImageProvider, model::AssetImage};

pub struct CoingeckoProvider {
    client: CoinGeckoClient,
    config: CoingeckoConfig,
}

impl CoingeckoProvider {
    pub fn new(client: CoinGeckoClient, config: CoingeckoConfig) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl ImageProvider for CoingeckoProvider {
    async fn get_asset_images(&self, coin_id: &str) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let coin_info = self.client.get_coin(coin_id).await?;
        Ok(mapper::map_platform_images(coin_info.platforms, coin_info.image.large))
    }
}

#[async_trait]
impl ImageListProvider for CoingeckoProvider {
    async fn get_top_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        if self.config.top.count == 0 {
            return Ok(vec![]);
        }

        let pages = self.config.top.count.div_ceil(MAX_MARKETS_PER_PAGE);
        let markets = self
            .client
            .get_all_coin_markets(None, MAX_MARKETS_PER_PAGE, pages)
            .await?
            .into_iter()
            .take(self.config.top.count)
            .collect();
        let coins = self.client.get_coin_list().await?;
        Ok(mapper::map_market_images(markets, mapper::coins_by_id(coins)))
    }

    async fn get_trending_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let trending = self.client.get_search_trending().await?;
        let coins = self.client.get_coin_list().await?;
        Ok(mapper::map_trending_images(trending, mapper::coins_by_id(coins)))
    }
}
