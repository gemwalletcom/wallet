use primitives::asset_score::AssetRank;
use std::error::Error;
use storage::{AssetFilter, AssetUpdate, AssetsRepository, Database, DatabaseError};

use crate::assets::AssetClassificationRules;

pub struct AssetRankUpdater {
    database: Database,
    classification_rules: AssetClassificationRules,
}

impl AssetRankUpdater {
    pub fn new(database: Database, classification_rules: AssetClassificationRules) -> Self {
        AssetRankUpdater { database, classification_rules }
    }

    pub async fn update_suspicious_assets(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let assets = self.database.run(|client| client.get_assets_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::RankLte(15)])).await?;
        let risks = assets
            .into_iter()
            .filter_map(|asset| self.classification_rules.classify(asset.score.rank, &asset.asset.name, &asset.asset.symbol).map(|risk| (asset.asset.id, risk)))
            .collect::<Vec<_>>();
        let spam = risks.iter().filter(|(_, rank)| *rank == AssetRank::Spam).map(|(asset_id, _)| asset_id.clone()).collect();
        let fraudulent = risks.into_iter().filter(|(_, rank)| *rank == AssetRank::Fraudulent).map(|(asset_id, _)| asset_id).collect();

        Ok(self
            .database
            .run(move |client| -> Result<usize, DatabaseError> {
                let spam_count = client.update_assets(spam, vec![AssetUpdate::Rank(AssetRank::Spam.threshold()), AssetUpdate::IsEnabled(false)])?;
                let fraudulent_count = client.update_assets(fraudulent, vec![AssetUpdate::Rank(AssetRank::Fraudulent.threshold()), AssetUpdate::IsEnabled(false)])?;
                Ok(spam_count + fraudulent_count)
            })
            .await?)
    }
}
