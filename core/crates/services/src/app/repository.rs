use async_trait::async_trait;
use primitives::{FeaturePolicy, Release};
use storage::{Database, DatabaseError, FeaturesRepository, ReleasesRepository};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn releases(&self) -> Result<Vec<Release>, DatabaseError>;
    async fn features(&self) -> Result<Vec<FeaturePolicy>, DatabaseError>;
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
    async fn releases(&self) -> Result<Vec<Release>, DatabaseError> {
        self.database.run(ReleasesRepository::get_releases).await
    }

    async fn features(&self) -> Result<Vec<FeaturePolicy>, DatabaseError> {
        self.database.run(FeaturesRepository::get_features).await
    }
}
