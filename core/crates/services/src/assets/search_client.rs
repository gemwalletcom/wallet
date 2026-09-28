use std::collections::HashMap;
use std::error::Error;

use primitives::{AssetBasic, AssetId, AssetList, NFTCollection, PerpetualSearchData};
use search_index::{ASSET_LISTS_INDEX_NAME, ASSETS_INDEX_NAME, AssetListDocument, NFTDocument, NFTS_INDEX_NAME, PERPETUALS_INDEX_NAME, PerpetualDocument, SearchIndexClient};

use super::search_filter::{build_assets_filters, build_filter, build_perpetuals_filters};
use super::search_request::SearchRequest;
use crate::prices::PriceClient;

pub struct SearchClient {
    client: SearchIndexClient,
    price_client: PriceClient,
}

impl SearchClient {
    pub fn new(client: SearchIndexClient, price_client: PriceClient) -> Self {
        Self { client, price_client }
    }

    pub async fn get_assets_search(&self, request: &SearchRequest) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        let filters = build_assets_filters(request);

        let assets: Vec<AssetBasic> = self.client.search(ASSETS_INDEX_NAME, &request.query, &build_filter(filters), [].as_ref(), request.limit, request.offset).await?;

        if assets.is_empty() {
            return Ok(vec![]);
        }

        let asset_ids: Vec<AssetId> = assets.iter().map(|asset| asset.asset.id.clone()).collect();
        let prices: HashMap<AssetId, _> = self.price_client.get_cache_prices(asset_ids).await?.into_iter().map(|price| (price.asset_id.clone(), price.as_price_primitive())).collect();

        Ok(assets
            .into_iter()
            .map(|asset| {
                let price = prices.get(&asset.asset.id).cloned();
                AssetBasic { price, ..asset }
            })
            .collect())
    }

    pub async fn get_asset_lists_search(&self, request: &SearchRequest) -> Result<Vec<AssetList>, Box<dyn Error + Send + Sync>> {
        let lists: Vec<AssetListDocument> = self.client.search(ASSET_LISTS_INDEX_NAME, &request.query, &build_filter(vec![]), [].as_ref(), request.limit, request.offset).await?;

        Ok(lists.iter().filter_map(|list| list.as_primitive(&request.chains)).collect())
    }

    pub async fn get_perpetuals_search(&self, request: &SearchRequest) -> Result<Vec<PerpetualSearchData>, Box<dyn Error + Send + Sync>> {
        let filters = build_perpetuals_filters(request);

        let perpetuals: Vec<PerpetualDocument> = self.client.search(PERPETUALS_INDEX_NAME, &request.query, &build_filter(filters), [].as_ref(), request.limit, request.offset).await?;

        Ok(perpetuals.into_iter().map(Into::into).collect())
    }

    pub async fn get_nfts_search(&self, request: &SearchRequest) -> Result<Vec<NFTCollection>, Box<dyn Error + Send + Sync>> {
        let nfts: Vec<NFTDocument> = self.client.search(NFTS_INDEX_NAME, &request.query, &build_filter(vec![]), [].as_ref(), request.limit, request.offset).await?;

        Ok(nfts.into_iter().map(|nft| nft.collection).collect())
    }
}
