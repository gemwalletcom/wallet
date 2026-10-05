use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use cacher::{CacheError, MarketsCacher};
use primitives::{AssetId, AssetTag, Markets, MarketsAssets, PriceId, PriceProvider};

use super::repository::Repository;

#[derive(Clone)]
pub struct MarketsClient {
    repository: Arc<dyn Repository>,
    cacher: Arc<dyn MarketsCacher>,
}

impl MarketsClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, cacher: Arc<dyn MarketsCacher>) -> Self {
        Self { repository, cacher }
    }

    pub async fn get_markets(&self) -> Result<Markets, Box<dyn Error + Send + Sync>> {
        match self.cacher.markets().await? {
            Some(markets) => Ok(markets),
            None => Err(Box::new(CacheError::not_found_resource("Markets"))),
        }
    }

    pub async fn set_markets(&self, markets: Markets) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.set_markets(&markets).await
    }

    pub async fn get_asset_ids_for_provider_price_ids(&self, provider: PriceProvider, provider_price_ids: Vec<String>) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        let price_ids: Vec<String> = provider_price_ids.iter().map(|id| PriceId::id_for(provider, id)).collect();
        let assets = self.repository.price_assets(price_ids.clone()).await?;
        let asset_map: HashMap<_, _> = assets.into_iter().map(|price_asset| (price_asset.price_id.to_string(), price_asset.asset_id)).collect();
        Ok(price_ids.into_iter().filter_map(|price_id| asset_map.get(&price_id).cloned()).collect())
    }

    pub async fn set_asset_ids_for_tag(&self, tag: AssetTag, asset_ids: Vec<AssetId>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.set_tag_asset_ids(tag.as_ref().to_string(), asset_ids).await?)
    }

    pub async fn get_asset_ids_for_tag(&self, tag: AssetTag) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.tag_asset_ids(vec![tag.as_ref().to_string()]).await?.into_iter().flatten().collect())
    }

    pub async fn get_market_assets(&self) -> Result<MarketsAssets, Box<dyn Error + Send + Sync>> {
        let tags = [AssetTag::Trending, AssetTag::Gainers, AssetTag::Losers].map(|tag| tag.as_ref().to_string()).to_vec();
        let [trending, gainers, losers]: [Vec<AssetId>; 3] = self.repository.tag_asset_ids(tags).await?.try_into().map_err(|_| "market tags query returned an unexpected shape")?;
        Ok(MarketsAssets { trending, gainers, losers })
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;
    use storage::PriceAsset;

    use super::*;
    use crate::testkit::MemoryPricesRepository;

    struct NoMarkets;

    #[async_trait::async_trait]
    impl MarketsCacher for NoMarkets {
        async fn markets(&self) -> Result<Option<Markets>, Box<dyn Error + Send + Sync>> {
            Ok(None)
        }

        async fn set_markets(&self, _markets: &Markets) -> Result<(), Box<dyn Error + Send + Sync>> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_asset_ids_follow_requested_price_ids() {
        let price_asset = |chain: Chain, id: &str| PriceAsset {
            asset_id: AssetId::from_chain(chain),
            price_id: PriceId::new(PriceProvider::Coingecko, id.to_string()),
        };
        let repository = MemoryPricesRepository::default().with_price_assets(vec![price_asset(Chain::Ethereum, "ethereum"), price_asset(Chain::Bitcoin, "bitcoin")]);
        let client = MarketsClient::new(Arc::new(repository), Arc::new(NoMarkets));

        let asset_ids = client
            .get_asset_ids_for_provider_price_ids(PriceProvider::Coingecko, vec!["bitcoin".to_string(), "unknown".to_string(), "ethereum".to_string()])
            .await
            .unwrap();

        assert_eq!(asset_ids, vec![AssetId::from_chain(Chain::Bitcoin), AssetId::from_chain(Chain::Ethereum)]);
    }
}
