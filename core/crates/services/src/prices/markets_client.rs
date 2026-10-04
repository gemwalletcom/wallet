use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use cacher::{CacheError, CacheKey, CacherClient};
use primitives::{AssetId, AssetTag, Markets, MarketsAssets, PriceId, PriceProvider};
use storage::{Database, DatabaseClient, DatabaseError, PricesRepository, TagRepository};

#[async_trait]
pub trait MarketsStore: Send + Sync {
    async fn markets(&self) -> Result<Option<Markets>, Box<dyn Error + Send + Sync>>;
    async fn set_markets(&self, markets: &Markets) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl MarketsStore for CacherClient {
    async fn markets(&self) -> Result<Option<Markets>, Box<dyn Error + Send + Sync>> {
        self.get_cached_optional(CacheKey::Markets).await
    }

    async fn set_markets(&self, markets: &Markets) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(CacheKey::Markets, markets).await
    }
}

#[derive(Clone)]
pub struct MarketsClient {
    database: Database,
    store: Arc<dyn MarketsStore>,
}

impl MarketsClient {
    pub fn new(database: Database, store: Arc<dyn MarketsStore>) -> Self {
        Self { database, store }
    }

    pub async fn get_markets(&self) -> Result<Markets, Box<dyn Error + Send + Sync>> {
        match self.store.markets().await? {
            Some(markets) => Ok(markets),
            None => Err(Box::new(CacheError::not_found_resource("Markets"))),
        }
    }

    pub async fn set_markets(&self, markets: Markets) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.store.set_markets(&markets).await
    }

    pub async fn get_asset_ids_for_provider_price_ids(&self, provider: PriceProvider, provider_price_ids: Vec<String>) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        let price_ids: Vec<String> = provider_price_ids.iter().map(|id| PriceId::id_for(provider, id)).collect();
        let lookup_price_ids = price_ids.clone();
        let assets = self.database.run(move |client| client.get_prices_assets_for_price_ids(lookup_price_ids)).await?;
        let asset_map: HashMap<_, _> = assets.into_iter().map(|price_asset| (price_asset.price_id.to_string(), price_asset.asset_id)).collect();
        Ok(price_ids.into_iter().filter_map(|price_id| asset_map.get(&price_id).cloned()).collect())
    }

    pub async fn set_asset_ids_for_tag(&self, tag: AssetTag, asset_ids: Vec<AssetId>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.run(move |client| client.set_assets_tags_for_tag(tag.as_ref(), asset_ids)).await?)
    }

    pub async fn get_asset_ids_for_tag(&self, tag: AssetTag) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.run(move |client| Self::asset_ids_for_tag(client, tag)).await?)
    }

    pub async fn get_market_assets(&self) -> Result<MarketsAssets, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .run(|client| -> Result<_, DatabaseError> {
                Ok(MarketsAssets {
                    trending: Self::asset_ids_for_tag(client, AssetTag::Trending)?,
                    gainers: Self::asset_ids_for_tag(client, AssetTag::Gainers)?,
                    losers: Self::asset_ids_for_tag(client, AssetTag::Losers)?,
                })
            })
            .await?)
    }

    fn asset_ids_for_tag(client: &mut DatabaseClient, tag: AssetTag) -> Result<Vec<AssetId>, DatabaseError> {
        client.get_asset_ids_for_tag(tag.as_ref())
    }
}
