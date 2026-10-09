use std::error::Error;
use std::sync::Arc;

use primitives::asset_score::AssetRank;

use crate::assets::AssetClassificationRules;
use crate::assets::repository::{RankChange, Repository};

pub struct AssetRankUpdater {
    repository: Arc<dyn Repository>,
    classification_rules: AssetClassificationRules,
}

impl AssetRankUpdater {
    pub(crate) fn new(repository: Arc<dyn Repository>, classification_rules: AssetClassificationRules) -> Self {
        AssetRankUpdater { repository, classification_rules }
    }

    pub async fn update_suspicious_assets(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let assets = self.repository.get_enabled_assets_at_or_below(AssetRank::Trivial).await?;
        let risks = assets
            .into_iter()
            .filter_map(|asset| self.classification_rules.classify(asset.score.rank, &asset.asset.name, &asset.asset.symbol).map(|risk| (asset.asset.id, risk)))
            .collect::<Vec<_>>();
        let spam = risks.iter().filter(|(_, rank)| *rank == AssetRank::Spam).map(|(asset_id, _)| asset_id.clone()).collect();
        let fraudulent = risks.into_iter().filter(|(_, rank)| *rank == AssetRank::Fraudulent).map(|(asset_id, _)| asset_id).collect();

        Ok(self
            .repository
            .update_asset_ranks(vec![
                RankChange { asset_ids: spam, rank: AssetRank::Spam },
                RankChange {
                    asset_ids: fraudulent,
                    rank: AssetRank::Fraudulent,
                },
            ])
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use primitives::Asset;

    use super::*;
    use crate::testkit::MemoryAssetRepository;

    #[tokio::test]
    async fn test_update_suspicious_assets() {
        let mut suspicious = Asset::mock_erc20().as_basic_primitive();
        suspicious.asset.name = "www.example.com".to_string();
        let suspicious_id = suspicious.asset.id.clone();
        let repository = Arc::new(MemoryAssetRepository::new(vec![suspicious, Asset::mock_btc().as_basic_primitive()]));
        let updater = AssetRankUpdater::new(repository.clone(), AssetClassificationRules::mock_with_spam_marker("www."));

        let count = updater.update_suspicious_assets().await.unwrap();

        assert_eq!(count, 1);
        assert_eq!(repository.ranks(), vec![AssetRank::Trivial]);
        assert_eq!(
            repository.changes(),
            vec![
                RankChange {
                    asset_ids: vec![suspicious_id],
                    rank: AssetRank::Spam,
                },
                RankChange {
                    asset_ids: vec![],
                    rank: AssetRank::Fraudulent,
                }
            ]
        );
    }
}
