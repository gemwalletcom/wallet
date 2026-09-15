use std::collections::{HashMap, HashSet};

use primitives::{AssetId, Chain, asset_score::AssetRank};
use search_index::{ASSET_LISTS_INDEX_NAME, AssetListDocument, SearchIndexClient};
use storage::{
    AssetFilter, AssetsRepository, Database, TagRepository,
    models::{AssetTagRow, PerpetualTagRow, TagRow},
};

pub struct AssetListsIndexUpdater {
    database: Database,
    search_index: SearchIndexClient,
}

impl AssetListsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        Self {
            database,
            search_index: search_index.clone(),
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let tags = [self.database.tag()?.get_asset_list_tags()?, self.database.tag()?.get_perpetual_list_tags()?].concat();
        let assets_tags = self.searchable_assets_tags(self.database.tag()?.get_assets_tags()?)?;
        let perpetuals_tags = self.database.tag()?.get_perpetuals_tags()?;
        let documents = Self::build_documents(tags, &assets_tags, &perpetuals_tags);

        self.search_index.replace_documents(ASSET_LISTS_INDEX_NAME, documents).await
    }

    fn searchable_assets_tags(&self, assets_tags: Vec<AssetTagRow>) -> Result<Vec<AssetTagRow>, Box<dyn std::error::Error + Send + Sync>> {
        let filters = vec![
            AssetFilter::Ids(assets_tags.iter().map(|tag| tag.asset_id.to_string()).collect()),
            AssetFilter::IsEnabled(true),
            AssetFilter::RankGt(AssetRank::Trivial.threshold()),
        ];
        let searchable_asset_ids: HashSet<AssetId> = self.database.assets()?.get_asset_ids_by_filter(filters)?.into_iter().collect();
        Ok(assets_tags.into_iter().filter(|tag| searchable_asset_ids.contains(&tag.asset_id)).collect())
    }

    fn build_documents(tags: Vec<TagRow>, assets_tags: &[AssetTagRow], perpetuals_tags: &[PerpetualTagRow]) -> Vec<AssetListDocument> {
        tags.into_iter()
            .filter(|tag| tag.visibility.is_public())
            .map(|tag| {
                let chain_counts = Self::chain_counts(&tag.id, assets_tags, perpetuals_tags);
                AssetListDocument::new(tag.id, tag.name, chain_counts)
            })
            .collect()
    }

    fn chain_counts(tag_id: &str, assets_tags: &[AssetTagRow], perpetuals_tags: &[PerpetualTagRow]) -> HashMap<String, u32> {
        let asset_chains = assets_tags.iter().filter(|tag| tag.tag_id == tag_id).map(|tag| tag.asset_id.chain);
        let perpetual_chains = perpetuals_tags.iter().filter(|tag| tag.tag_id == tag_id).map(|_| Chain::HyperCore);
        asset_chains.chain(perpetual_chains).fold(HashMap::new(), |mut counts, chain| {
            *counts.entry(chain.to_string()).or_default() += 1;
            counts
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{PerpetualId, PerpetualProvider};
    use storage::sql_types::TagVisibility;

    fn tag(id: &str, visibility: TagVisibility) -> TagRow {
        TagRow {
            id: id.to_string(),
            name: id.to_string(),
            visibility,
            list_id: None,
        }
    }

    #[test]
    fn test_build_documents() {
        let tags = vec![
            tag("stablecoins", TagVisibility::Public),
            tag("stocks", TagVisibility::Public),
            tag("internal", TagVisibility::Internal),
        ];
        let assets_tags = vec![
            AssetTagRow::mock_with_tag(AssetId::from_chain(Chain::Ethereum), "stablecoins"),
            AssetTagRow::mock_with_tag(AssetId::from_token(Chain::Ethereum, "0x1"), "stablecoins"),
            AssetTagRow::mock_with_tag(AssetId::from_token(Chain::Solana, "abc"), "stablecoins"),
            AssetTagRow::mock_with_tag(AssetId::from_chain(Chain::Bitcoin), "internal"),
        ];
        let perpetuals_tags = vec![PerpetualTagRow {
            perpetual_id: PerpetualId::new(PerpetualProvider::Hypercore, "TSLA").into(),
            tag_id: "stocks".to_string(),
            order: None,
        }];

        let documents = AssetListsIndexUpdater::build_documents(tags, &assets_tags, &perpetuals_tags);

        assert_eq!(
            documents,
            vec![
                AssetListDocument::new(
                    "stablecoins".to_string(),
                    "stablecoins".to_string(),
                    HashMap::from([("ethereum".to_string(), 2), ("solana".to_string(), 1)]),
                ),
                AssetListDocument::new("stocks".to_string(), "stocks".to_string(), HashMap::from([("hypercore".to_string(), 1)])),
            ]
        );
    }
}
