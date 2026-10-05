use std::error::Error;
use std::sync::Mutex;

use async_trait::async_trait;
use primitives::{AssetBasic, AssetList, NFTCollection, PerpetualSearchData};
use search_index::{SearchProvider, SearchQuery};

pub(crate) struct MemorySearchProvider {
    assets: Vec<AssetBasic>,
    failure: Option<String>,
    queries: Mutex<Vec<(SearchQuery, i32)>>,
}

impl MemorySearchProvider {
    pub(crate) fn new(assets: Vec<AssetBasic>) -> Self {
        Self {
            assets,
            failure: None,
            queries: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn failing(message: &str) -> Self {
        Self {
            assets: Vec::new(),
            failure: Some(message.to_string()),
            queries: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn asset_queries(&self) -> Vec<(SearchQuery, i32)> {
        self.queries.lock().unwrap().clone()
    }

    fn check(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        match &self.failure {
            Some(message) => Err(message.clone().into()),
            None => Ok(()),
        }
    }
}

#[async_trait]
impl SearchProvider for MemorySearchProvider {
    async fn search_assets(&self, query: &SearchQuery, rank_threshold: i32) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        self.check()?;
        self.queries.lock().unwrap().push((query.clone(), rank_threshold));
        Ok(self.assets.clone())
    }

    async fn search_asset_lists(&self, _query: &SearchQuery) -> Result<Vec<AssetList>, Box<dyn Error + Send + Sync>> {
        self.check()?;
        Ok(vec![])
    }

    async fn search_perpetuals(&self, _query: &SearchQuery) -> Result<Vec<PerpetualSearchData>, Box<dyn Error + Send + Sync>> {
        self.check()?;
        Ok(vec![])
    }

    async fn search_nfts(&self, _query: &SearchQuery) -> Result<Vec<NFTCollection>, Box<dyn Error + Send + Sync>> {
        self.check()?;
        Ok(vec![])
    }
}
