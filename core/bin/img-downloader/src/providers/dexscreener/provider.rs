use std::{collections::HashSet, error::Error, time::Duration};

use async_trait::async_trait;
use dexscreener::{DexScreenerClient, chain_id};
use gem_tracing::info_with_fields;
use primitives::AssetId;
use tokio::time::sleep;

use super::mapper;
use crate::config::DexScreenerConfig;
use crate::providers::{ImageListProvider, ImageProvider, model::AssetImage};

const META_REQUEST_INTERVAL: Duration = Duration::from_secs(1);

pub struct DexScreenerProvider {
    client: DexScreenerClient,
    config: DexScreenerConfig,
}

impl DexScreenerProvider {
    pub fn new(client: DexScreenerClient, config: DexScreenerConfig) -> Self {
        Self { client, config }
    }

    async fn get_meta_images(&self, count: usize) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let metas = self.client.get_trending_metas().await?;
        let mut slugs = HashSet::new();
        let mut pairs = Vec::new();
        for meta in metas.into_iter().filter(|meta| slugs.insert(meta.slug.clone())) {
            sleep(META_REQUEST_INTERVAL).await;
            info_with_fields!("collect dexscreener meta", slug = meta.slug.as_str());
            pairs.extend(self.client.get_meta(&meta.slug).await?.pairs);
        }
        Ok(mapper::map_ranked_images(pairs, count))
    }
}

#[async_trait]
impl ImageProvider for DexScreenerProvider {
    async fn get_asset_images(&self, id: &str) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        let asset_id = AssetId::new(id).ok_or("Invalid asset ID")?;
        let chain = chain_id(asset_id.chain).ok_or("Unsupported DexScreener chain")?;
        let pairs = self.client.get_token_pairs(chain, asset_id.get_token_id()?).await?;
        Ok(mapper::map_asset_image(&asset_id, pairs).into_iter().collect())
    }
}

#[async_trait]
impl ImageListProvider for DexScreenerProvider {
    async fn get_top_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        self.get_meta_images(self.config.top.count).await
    }

    async fn get_trending_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>> {
        self.get_meta_images(self.config.trending.count).await
    }
}
