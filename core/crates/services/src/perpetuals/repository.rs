use async_trait::async_trait;
use primitives::perpetual::PerpetualAddress;
use storage::{Database, DatabaseError, PerpetualsAddressesRepository};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn set_perpetual_addresses(&self, addresses: Vec<PerpetualAddress>) -> Result<usize, DatabaseError>;
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
    async fn set_perpetual_addresses(&self, addresses: Vec<PerpetualAddress>) -> Result<usize, DatabaseError> {
        self.database.transaction(move |client| client.set_perpetual_addresses(addresses)).await
    }
}
