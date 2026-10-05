use std::collections::HashMap;
use std::sync::Arc;

use primitives::{Chain, asset_score::AssetRank};
use search_index::{ASSET_LISTS_INDEX_NAME, AssetListDocument, SearchIndexClient};
use storage::{AssetFilter, AssetTagLink, PerpetualTagLink, Tag};

use super::repository::{AssetListsIndexData, Repository};

pub struct AssetListsIndexUpdater {
    repository: Arc<dyn Repository>,
    search_index: SearchIndexClient,
}

impl AssetListsIndexUpdater {
    pub(crate) fn new(repository: Arc<dyn Repository>, search_index: SearchIndexClient) -> Self {
        Self { repository, search_index }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let AssetListsIndexData {
            tags,
            assets_tags,
            searchable_asset_ids,
            perpetuals_tags,
        } = self.repository.asset_lists(vec![AssetFilter::IsEnabled(true), AssetFilter::RankGt(AssetRank::Trivial.threshold())]).await?;
        let assets_tags = assets_tags.into_iter().filter(|tag| searchable_asset_ids.contains(&tag.asset_id)).collect::<Vec<_>>();
        let documents = Self::build_documents(tags, &assets_tags, &perpetuals_tags);

        self.search_index.replace_documents(ASSET_LISTS_INDEX_NAME, documents).await
    }

    fn build_documents(tags: Vec<Tag>, assets_tags: &[AssetTagLink], perpetuals_tags: &[PerpetualTagLink]) -> Vec<AssetListDocument> {
        tags.into_iter()
            .filter(Tag::is_public)
            .map(|tag| {
                let chain_counts = Self::chain_counts(&tag.id, assets_tags, perpetuals_tags);
                AssetListDocument::new(tag.id, tag.name, chain_counts)
            })
            .collect()
    }

    fn chain_counts(tag_id: &str, assets_tags: &[AssetTagLink], perpetuals_tags: &[PerpetualTagLink]) -> HashMap<String, u32> {
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
    use primitives::{AssetId, PerpetualId, PerpetualProvider, TagVisibility};

    fn asset_tag(asset_id: AssetId, tag_id: &str) -> AssetTagLink {
        AssetTagLink { asset_id, tag_id: tag_id.to_string() }
    }

    #[test]
    fn test_build_documents() {
        let tags = vec![Tag::mock("stablecoins", TagVisibility::Public), Tag::mock("stocks", TagVisibility::Public), Tag::mock("internal", TagVisibility::Internal)];
        let assets_tags = vec![
            asset_tag(AssetId::from_chain(Chain::Ethereum), "stablecoins"),
            asset_tag(AssetId::from_token(Chain::Ethereum, "0x1"), "stablecoins"),
            asset_tag(AssetId::from_token(Chain::Solana, "abc"), "stablecoins"),
            asset_tag(AssetId::from_chain(Chain::Bitcoin), "internal"),
        ];
        let perpetuals_tags = vec![PerpetualTagLink {
            perpetual_id: PerpetualId::new(PerpetualProvider::Hypercore, "TSLA"),
            tag_id: "stocks".to_string(),
        }];

        let documents = AssetListsIndexUpdater::build_documents(tags, &assets_tags, &perpetuals_tags);

        assert_eq!(
            documents,
            vec![
                AssetListDocument::new("stablecoins".to_string(), "stablecoins".to_string(), HashMap::from([("ethereum".to_string(), 2), ("solana".to_string(), 1)]),),
                AssetListDocument::new("stocks".to_string(), "stocks".to_string(), HashMap::from([("hypercore".to_string(), 1)])),
            ]
        );
    }
}
