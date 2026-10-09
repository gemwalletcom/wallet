use async_trait::async_trait;
use primitives::ChainAddress;
use storage::{Database, DatabaseError, WalletsRepository};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn get_wallet_subscriptions(&self, device_id: i32, wallet_id: i32) -> Result<Vec<ChainAddress>, DatabaseError>;
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
    async fn get_wallet_subscriptions(&self, device_id: i32, wallet_id: i32) -> Result<Vec<ChainAddress>, DatabaseError> {
        self.database.run(move |client| client.get_subscriptions_by_wallet_id(device_id, wallet_id)).await
    }
}
