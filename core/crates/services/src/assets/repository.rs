use async_trait::async_trait;
use primitives::{AssetBasic, AssetId, asset_score::AssetRank};
use storage::{AssetFilter, AssetUpdate, AssetsRepository, Database, DatabaseError};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RankChange {
    pub(crate) asset_ids: Vec<AssetId>,
    pub(crate) rank: AssetRank,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn enabled_assets_at_or_below(&self, rank: AssetRank) -> Result<Vec<AssetBasic>, DatabaseError>;
    async fn disable_assets_with_ranks(&self, changes: Vec<RankChange>) -> Result<usize, DatabaseError>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

impl PostgresRepository {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn enabled_assets_at_or_below(&self, rank: AssetRank) -> Result<Vec<AssetBasic>, DatabaseError> {
        self.database.run(move |client| client.get_assets_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::RankLte(rank.threshold())])).await
    }

    async fn disable_assets_with_ranks(&self, changes: Vec<RankChange>) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                changes.into_iter().try_fold(0, |count, change| {
                    let updated = client.update_assets(change.asset_ids, vec![AssetUpdate::Rank(change.rank.threshold()), AssetUpdate::IsEnabled(false)])?;
                    Ok::<_, DatabaseError>(count + updated)
                })
            })
            .await
    }
}
