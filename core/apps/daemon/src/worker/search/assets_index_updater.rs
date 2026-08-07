use std::collections::HashMap;
use std::time::Duration;

use super::sync::{SearchSyncClient, SearchSyncResult};
use primitives::ConfigKey;
use search_index::{ASSETS_INDEX_NAME, AssetDocument, SearchIndexClient, sanitize_index_primary_id};
use storage::models::{AssetTagRow, PriceAssetDataRow};
use storage::{AssetsUsageRanksRepository, AssetsWithPricesFilter, Database, PricesRepository, TagRepository};

pub struct AssetsIndexUpdater {
    database: Database,
    sync_client: SearchSyncClient,
    primary_price_max_age: Duration,
}

impl AssetsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient, primary_price_max_age: Duration) -> Self {
        Self {
            sync_client: SearchSyncClient::new(database.clone(), search_index),
            database,
            primary_price_max_age,
        }
    }

    pub async fn update(&self) -> Result<SearchSyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let sync = self.sync_client.for_key(ConfigKey::SearchAssetsLastUpdatedAt)?;
        let filters = sync.since().map(AssetsWithPricesFilter::UpdatedSince).into_iter().collect();
        let prices = PricesRepository::get_assets_with_prices(&mut self.database.prices()?, filters, self.primary_price_max_age)?;

        if prices.is_empty() {
            return sync.write(ASSETS_INDEX_NAME, Vec::<AssetDocument>::new()).await;
        }

        let usage_ranks = self.database.assets_usage_ranks()?.get_all_usage_ranks()?;
        let assets_tags_map = Self::asset_tags_by_asset(self.database.tag()?.get_assets_tags()?);
        let usage_ranks_map: HashMap<String, i32> = usage_ranks.into_iter().map(|r| (r.asset_id.to_string(), r.usage_rank)).collect();

        let documents = Self::build_documents(prices.iter(), &assets_tags_map, &usage_ranks_map);

        sync.write(ASSETS_INDEX_NAME, documents).await
    }

    fn build_documents<'a>(
        prices: impl IntoIterator<Item = &'a PriceAssetDataRow>,
        assets_tags_map: &HashMap<String, Vec<String>>,
        usage_ranks_map: &HashMap<String, i32>,
    ) -> Vec<AssetDocument> {
        prices
            .into_iter()
            .map(|x| {
                let asset_id = x.asset.id.as_str();
                let asset = x.asset.as_primitive();
                let usage_rank = usage_ranks_map.get(asset_id).copied().unwrap_or(0);
                AssetDocument {
                    id: sanitize_index_primary_id(asset_id),
                    aliases: AssetDocument::aliases(&asset),
                    asset,
                    properties: x.asset.clone().as_property_primitive(),
                    score: x.asset.clone().as_score_primitive(),
                    usage_rank,
                    market: x.price.as_ref().map(|price| price.as_market_primitive(&x.asset)),
                    tags: assets_tags_map.get(asset_id).cloned(),
                }
            })
            .collect()
    }

    fn asset_tags_by_asset(tags: impl IntoIterator<Item = AssetTagRow>) -> HashMap<String, Vec<String>> {
        tags.into_iter().fold(HashMap::new(), |mut acc, tag| {
            acc.entry(tag.asset_id.to_string()).or_default().push(tag.tag_id);
            acc
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain};

    #[test]
    fn asset_tags_by_asset_includes_internal_tags() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let tags = vec![
            AssetTagRow::mock_with_tag(asset_id.clone(), "trending"),
            AssetTagRow::mock_with_tag(asset_id.clone(), "stablecoins"),
        ];

        let tags_by_asset = AssetsIndexUpdater::asset_tags_by_asset(tags);

        assert_eq!(tags_by_asset.get(&asset_id.to_string()), Some(&vec!["trending".to_string(), "stablecoins".to_string()]));
    }
}
