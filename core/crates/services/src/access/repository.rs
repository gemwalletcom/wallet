use async_trait::async_trait;
use storage::{ApiClientResource, ApiClientScope, ApiClientsRepository, Database, DatabaseError};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn has_enabled_api_client(&self, secret: String, scope: ApiClientScope, resource: ApiClientResource) -> Result<bool, DatabaseError>;
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
    async fn has_enabled_api_client(&self, secret: String, scope: ApiClientScope, resource: ApiClientResource) -> Result<bool, DatabaseError> {
        self.database.run(move |client| client.has_enabled_api_client(&secret, scope, resource)).await
    }
}
