use async_trait::async_trait;
use storage::{AssetsRepository, Database, DatabaseError};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn swap_asset_ids(&self) -> Result<Vec<String>, DatabaseError>;
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
    async fn swap_asset_ids(&self) -> Result<Vec<String>, DatabaseError> {
        self.database.run(AssetsRepository::get_swap_assets).await
    }
}
