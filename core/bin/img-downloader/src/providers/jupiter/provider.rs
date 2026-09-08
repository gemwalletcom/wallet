use std::error::Error;

use ::jupiter::JupiterClient;
use async_trait::async_trait;

use super::mapper;
use crate::config::JupiterConfig;
use crate::providers::{ImageListProvider, ImageProvider, model::AssetImage};

pub struct JupiterProvider {
    client: JupiterClient,
    config: JupiterConfig,
}

impl JupiterProvider {
    pub fn new(client: JupiterClient, config: JupiterConfig) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl ImageProvider for JupiterProvider {
    async fn get_asset_images(&self, token_id: &str) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let tokens = self.client.get_verified_tokens().await?;
        Ok(mapper::map_tokens(tokens).into_iter().filter(|image| image.token_id == token_id).collect())
    }
}

#[async_trait]
impl ImageListProvider for JupiterProvider {
    async fn get_top_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let tokens = self.client.get_verified_tokens().await?;
        Ok(mapper::map_tokens(tokens.into_iter().take(self.config.top.count).collect()))
    }

    async fn get_trending_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let tokens = self.client.get_top_trending_tokens(&self.config.trending.interval, self.config.trending.count).await?;
        Ok(mapper::map_tokens(tokens))
    }
}
