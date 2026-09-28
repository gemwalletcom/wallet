use std::collections::HashMap;
use std::sync::Arc;

use super::sync::{SearchSyncClient, SearchSyncResult};
use crate::ConfigCacher;
use config_keys::ConfigKey;
use search_index::{ASSETS_INDEX_NAME, AssetDocument, SearchIndexClient, sanitize_index_primary_id};
use storage::{AssetTagLink, AssetWithMarket, AssetsUsageRanksRepository, AssetsWithPricesFilter, Database, DatabaseError, PricesRepository, TagRepository};

pub struct AssetsIndexUpdater {
    database: Database,
    sync_client: SearchSyncClient,
    config: Arc<ConfigCacher>,
}

impl AssetsIndexUpdater {
    pub fn new(database: Database, config: Arc<ConfigCacher>, search_index: &SearchIndexClient) -> Self {
        Self {
            sync_client: SearchSyncClient::new(config.clone(), search_index),
            database,
            config,
        }
    }

    pub async fn update(&self) -> Result<SearchSyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let sync = self.sync_client.for_key(ConfigKey::SearchAssetsLastUpdatedAt).await?;
        let filters = sync.since().map(AssetsWithPricesFilter::UpdatedSince).into_iter().collect();
        let primary_price_max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        let assets = self.database.run(move |client| client.get_assets_markets(filters, primary_price_max_age)).await?;

        if assets.is_empty() {
            return sync.write(ASSETS_INDEX_NAME, Vec::<AssetDocument>::new()).await;
        }

        let (usage_ranks, assets_tags) = self.database.run(|client| -> Result<_, DatabaseError> { Ok((client.get_all_usage_ranks()?, client.get_assets_tags()?)) }).await?;
        let assets_tags_map = Self::asset_tags_by_asset(assets_tags);
        let usage_ranks_map: HashMap<String, i32> = usage_ranks.into_iter().map(|(asset_id, usage_rank)| (asset_id.to_string(), usage_rank)).collect();

        let documents = Self::build_documents(assets, &assets_tags_map, &usage_ranks_map);

        sync.write(ASSETS_INDEX_NAME, documents).await
    }

    fn build_documents(assets: Vec<AssetWithMarket>, assets_tags_map: &HashMap<String, Vec<String>>, usage_ranks_map: &HashMap<String, i32>) -> Vec<AssetDocument> {
        assets
            .into_iter()
            .map(|AssetWithMarket { asset, market }| {
                let asset_id = asset.asset.id.to_string();
                let usage_rank = usage_ranks_map.get(&asset_id).copied().unwrap_or(0);
                AssetDocument {
                    id: sanitize_index_primary_id(&asset_id),
                    aliases: AssetDocument::aliases(&asset.asset),
                    chain: asset.asset.chain(),
                    token_id: asset.asset.token_id().map(str::to_string),
                    tags: assets_tags_map.get(&asset_id).cloned(),
                    asset: asset.asset,
                    properties: asset.properties,
                    score: asset.score,
                    usage_rank,
                    market,
                }
            })
            .collect()
    }

    fn asset_tags_by_asset(tags: impl IntoIterator<Item = AssetTagLink>) -> HashMap<String, Vec<String>> {
        tags.into_iter().fold(HashMap::new(), |mut acc, tag| {
            acc.entry(tag.asset_id.to_string()).or_default().push(tag.tag_id);
            acc
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, AssetId, AssetType, Chain};
    use serde_json::Value;

    #[test]
    fn test_build_documents_preserves_chain_and_token_search_fields() {
        let token_id = "0xbe9D156892E55e7154BcD3cB0FEA677F9D3103E1";
        let assets = [AssetId::token(Chain::SmartChain, token_id), AssetId::from_chain(Chain::Ethereum)]
            .into_iter()
            .map(|id| AssetWithMarket {
                asset: Asset::new(id, "Asset".to_string(), "ASSET".to_string(), 18, AssetType::ERC20).as_basic_primitive(),
                market: None,
            })
            .collect::<Vec<_>>();

        let documents = AssetsIndexUpdater::build_documents(assets, &HashMap::new(), &HashMap::new());
        let indexed = serde_json::to_value(documents).unwrap();

        assert_eq!(indexed[0]["chain"].as_str(), Some("smartchain"));
        assert_eq!(indexed[0]["tokenId"].as_str(), Some(token_id));
        assert_eq!(indexed[1]["chain"].as_str(), Some("ethereum"));
        assert_eq!(indexed[1]["tokenId"], Value::Null);
    }

    #[test]
    fn asset_tags_by_asset_includes_internal_tags() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let tag = |tag_id: &str| AssetTagLink {
            asset_id: asset_id.clone(),
            tag_id: tag_id.to_string(),
        };
        let tags = vec![tag("trending"), tag("stablecoins")];

        let tags_by_asset = AssetsIndexUpdater::asset_tags_by_asset(tags);

        assert_eq!(tags_by_asset.get(&asset_id.to_string()), Some(&vec!["trending".to_string(), "stablecoins".to_string()]));
    }
}
