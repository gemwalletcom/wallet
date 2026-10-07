use std::error::Error;

use async_trait::async_trait;
use primitives::{AssetBasic, AssetList, NFTCollection, PerpetualSearchData};
use serde_json::Value;

use crate::{ASSET_LISTS_INDEX_NAME, ASSETS_INDEX_NAME, AssetListDocument, NFTDocument, NFTS_INDEX_NAME, PERPETUALS_INDEX_NAME, PerpetualDocument, SearchIndexClient};

#[derive(Debug, Clone, PartialEq)]
pub struct SearchQuery {
    pub query: String,
    pub chains: Vec<String>,
    pub tags: Vec<String>,
    pub limit: usize,
    pub offset: usize,
}

#[async_trait]
pub trait SearchProvider: Send + Sync {
    async fn search_assets(&self, query: &SearchQuery, rank_threshold: i32) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>>;
    async fn search_asset_lists(&self, query: &SearchQuery) -> Result<Vec<AssetList>, Box<dyn Error + Send + Sync>>;
    async fn search_perpetuals(&self, query: &SearchQuery) -> Result<Vec<PerpetualSearchData>, Box<dyn Error + Send + Sync>>;
    async fn search_nfts(&self, query: &SearchQuery) -> Result<Vec<NFTCollection>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl SearchProvider for SearchIndexClient {
    async fn search_assets(&self, query: &SearchQuery, rank_threshold: i32) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        self.search(ASSETS_INDEX_NAME, &query.query, &build_filter(assets_filters(query, rank_threshold)), &[], query.limit, query.offset).await
    }

    async fn search_asset_lists(&self, query: &SearchQuery) -> Result<Vec<AssetList>, Box<dyn Error + Send + Sync>> {
        let lists: Vec<AssetListDocument> = self.search(ASSET_LISTS_INDEX_NAME, &query.query, &build_filter(vec![]), &[], query.limit, query.offset).await?;
        Ok(lists.iter().filter_map(|list| list.as_primitive(&query.chains)).collect())
    }

    async fn search_perpetuals(&self, query: &SearchQuery) -> Result<Vec<PerpetualSearchData>, Box<dyn Error + Send + Sync>> {
        let perpetuals: Vec<PerpetualDocument> = self.search(PERPETUALS_INDEX_NAME, &query.query, &build_filter(perpetuals_filters(query)), &[], query.limit, query.offset).await?;
        Ok(perpetuals.into_iter().map(Into::into).collect())
    }

    async fn search_nfts(&self, query: &SearchQuery) -> Result<Vec<NFTCollection>, Box<dyn Error + Send + Sync>> {
        let nfts: Vec<NFTDocument> = self.search(NFTS_INDEX_NAME, &query.query, &build_filter(vec![]), &[], query.limit, query.offset).await?;
        Ok(nfts.into_iter().map(|nft| nft.collection).collect())
    }
}

fn assets_filters(query: &SearchQuery, rank_threshold: i32) -> Vec<String> {
    let mut filters = vec!["properties.isEnabled = true".to_string(), format!("score.rank > {rank_threshold}")];
    if !query.tags.is_empty() {
        filters.push(filter_array("tags", &query.tags));
    }
    if !query.chains.is_empty() {
        filters.push(filter_array("chain", &query.chains));
    }
    filters
}

fn perpetuals_filters(query: &SearchQuery) -> Vec<String> {
    if query.tags.is_empty() { vec![] } else { vec![filter_array("tags", &query.tags)] }
}

fn build_filter(filters: Vec<String>) -> String {
    filters.join(" AND ")
}

fn filter_array(field: &str, values: &[String]) -> String {
    let values = values.iter().map(|value| Value::String(value.clone()).to_string()).collect::<Vec<_>>().join(",");
    format!("{field} IN [{values}]")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(chains: &[&str], tags: &[&str]) -> SearchQuery {
        SearchQuery {
            query: "query".to_string(),
            chains: chains.iter().map(ToString::to_string).collect(),
            tags: tags.iter().map(ToString::to_string).collect(),
            limit: 100,
            offset: 0,
        }
    }

    #[test]
    fn test_assets_filters() {
        assert_eq!(assets_filters(&query(&[], &[]), 15), vec!["properties.isEnabled = true", "score.rank > 15"]);
        assert_eq!(assets_filters(&query(&[], &["defi"]), 5), vec!["properties.isEnabled = true", "score.rank > 5", "tags IN [\"defi\"]"]);
        assert_eq!(assets_filters(&query(&["ethereum"], &[]), 5), vec!["properties.isEnabled = true", "score.rank > 5", "chain IN [\"ethereum\"]"]);
        assert_eq!(
            assets_filters(&query(&["smartchain"], &["bstocks"]), 15),
            vec!["properties.isEnabled = true", "score.rank > 15", "tags IN [\"bstocks\"]", "chain IN [\"smartchain\"]"]
        );
    }

    #[test]
    fn test_perpetuals_filters() {
        assert!(perpetuals_filters(&query(&[], &[])).is_empty());
        assert_eq!(perpetuals_filters(&query(&[], &["stocks"])), vec!["tags IN [\"stocks\"]"]);
    }

    #[test]
    fn test_build_filter() {
        assert_eq!(build_filter(vec!["a".to_string(), "b".to_string()]), "a AND b");
        assert_eq!(build_filter(vec![]), "");
    }

    #[test]
    fn test_filter_array() {
        assert_eq!(filter_array("tags", &["defi".to_string(), "nft".to_string()]), "tags IN [\"defi\",\"nft\"]");
        assert_eq!(
            filter_array("tags", &["x\"] OR properties.isEnabled = false OR tags IN [\"y".to_string()]),
            r#"tags IN ["x\"] OR properties.isEnabled = false OR tags IN [\"y"]"#
        );
    }
}
