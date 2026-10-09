use async_trait::async_trait;
use primitives::{Chain, DeviceSubscription};
use storage::{Database, DatabaseError, WalletsRepository};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn get_subscriptions_for_addresses(&self, chain: Chain, addresses: Vec<String>) -> Result<Vec<DeviceSubscription>, DatabaseError>;
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
    async fn get_subscriptions_for_addresses(&self, chain: Chain, addresses: Vec<String>) -> Result<Vec<DeviceSubscription>, DatabaseError> {
        self.database.run(move |client| client.get_subscriptions_by_chain_addresses(chain, addresses)).await
    }
}
