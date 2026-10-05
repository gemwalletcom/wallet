use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use cacher::PriceCacher;
use primitives::{AssetBasic, AssetId, AssetList, NFTCollection, PerpetualSearchData};
use search_index::SearchProvider;

use super::search_request::SearchRequest;

pub struct SearchClient {
    search: Arc<dyn SearchProvider>,
    prices: Arc<dyn PriceCacher>,
}

impl SearchClient {
    pub fn new(search: Arc<dyn SearchProvider>, prices: Arc<dyn PriceCacher>) -> Self {
        Self { search, prices }
    }

    pub async fn get_assets_search(&self, request: &SearchRequest) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        let assets = self.search.search_assets(&request.search_query(), request.rank_threshold()).await?;

        if assets.is_empty() {
            return Ok(vec![]);
        }

        let asset_ids: Vec<AssetId> = assets.iter().map(|asset| asset.asset.id.clone()).collect();
        let prices: HashMap<AssetId, _> = self.prices.prices(&asset_ids).await?.into_iter().map(|price| (price.asset_id.clone(), price.as_price_primitive())).collect();

        Ok(assets
            .into_iter()
            .map(|asset| {
                let price = prices.get(&asset.asset.id).cloned();
                AssetBasic { price, ..asset }
            })
            .collect())
    }

    pub async fn get_asset_lists_search(&self, request: &SearchRequest) -> Result<Vec<AssetList>, Box<dyn Error + Send + Sync>> {
        self.search.search_asset_lists(&request.search_query()).await
    }

    pub async fn get_perpetuals_search(&self, request: &SearchRequest) -> Result<Vec<PerpetualSearchData>, Box<dyn Error + Send + Sync>> {
        self.search.search_perpetuals(&request.search_query()).await
    }

    pub async fn get_nfts_search(&self, request: &SearchRequest) -> Result<Vec<NFTCollection>, Box<dyn Error + Send + Sync>> {
        self.search.search_nfts(&request.search_query()).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use primitives::{Asset, AssetMarket, AssetPriceInfo, Chain, MAX_QUERY_LIMIT, Price, PriceProvider};

    use super::*;
    use crate::testkit::{MemoryPriceCacher, MemorySearchProvider};

    fn price_info(chain: Chain, price: f64) -> AssetPriceInfo {
        AssetPriceInfo {
            asset_id: AssetId::from_chain(chain),
            price: Price::new(price, 1.0, Utc::now(), PriceProvider::Coingecko),
            market: AssetMarket::mock(),
        }
    }

    #[tokio::test]
    async fn test_get_assets_search_adds_cached_prices() {
        let search = Arc::new(MemorySearchProvider::new(vec![Asset::from_chain(Chain::Ethereum).as_basic_primitive(), Asset::from_chain(Chain::Bitcoin).as_basic_primitive()]));
        let client = SearchClient::new(search.clone(), Arc::new(MemoryPriceCacher::new(vec![price_info(Chain::Ethereum, 2000.0)])));
        let request = SearchRequest::new("ethereum contract", Some("ethereum"), None, MAX_QUERY_LIMIT, None);

        let assets = client.get_assets_search(&request).await.unwrap();

        assert_eq!(assets.iter().map(|asset| asset.price.map(|price| price.price)).collect::<Vec<_>>(), vec![Some(2000.0), None]);
        assert_eq!(search.asset_queries(), vec![(request.search_query(), 5)]);
    }

    #[tokio::test]
    async fn test_get_assets_search_without_results_skips_prices() {
        let prices = Arc::new(MemoryPriceCacher::new(vec![]));
        let client = SearchClient::new(Arc::new(MemorySearchProvider::new(vec![])), prices.clone());

        let assets = client.get_assets_search(&SearchRequest::new("BTC", None, None, MAX_QUERY_LIMIT, None)).await.unwrap();

        assert!(assets.is_empty());
        assert!(prices.requests().is_empty());
    }

    #[tokio::test]
    async fn test_get_assets_search_failure() {
        let prices = Arc::new(MemoryPriceCacher::new(vec![]));
        let client = SearchClient::new(Arc::new(MemorySearchProvider::failing("meilisearch unavailable")), prices.clone());

        let error = client.get_assets_search(&SearchRequest::new("BTC", None, None, MAX_QUERY_LIMIT, None)).await.unwrap_err();

        assert_eq!(error.to_string(), "meilisearch unavailable");
        assert!(prices.requests().is_empty());
    }
}
