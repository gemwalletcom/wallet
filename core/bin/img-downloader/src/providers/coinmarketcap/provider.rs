use std::error::Error;

use async_trait::async_trait;
use coinmarketcap::{CoinMarketCapClient, Listing};

use super::mapper;
use crate::config::CoinMarketCapConfig;
use crate::providers::{ImageListProvider, ImageProvider, model::AssetImage};

pub struct CoinMarketCapProvider {
    client: CoinMarketCapClient,
    config: CoinMarketCapConfig,
}

impl CoinMarketCapProvider {
    pub fn new(client: CoinMarketCapClient, config: CoinMarketCapConfig) -> Self {
        Self { client, config }
    }

    async fn get_asset_images_by_ids(&self, ids: Vec<u64>) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let mut images = Vec::new();
        for chunk in ids.chunks(100) {
            images.extend(self.client.get_info_by_ids(chunk).await?.into_iter().flat_map(mapper::map_info));
        }
        Ok(images)
    }
}

#[async_trait]
impl ImageProvider for CoinMarketCapProvider {
    async fn get_asset_images(&self, id_or_symbol: &str) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_info_by_id_or_symbol(id_or_symbol).await?.into_iter().flat_map(mapper::map_info).collect())
    }
}

#[async_trait]
impl ImageListProvider for CoinMarketCapProvider {
    async fn get_top_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let ids = self
            .client
            .get_latest_listings(self.config.top.count)
            .await?
            .into_iter()
            .filter(Listing::is_token)
            .map(|listing| listing.id)
            .collect();
        self.get_asset_images_by_ids(ids).await
    }

    async fn get_trending_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let ids = self
            .client
            .get_trending_latest(self.config.trending.count)
            .await?
            .into_iter()
            .filter(Listing::is_token)
            .map(|listing| listing.id)
            .collect();
        self.get_asset_images_by_ids(ids).await
    }
}
