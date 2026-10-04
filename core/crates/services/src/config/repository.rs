use async_trait::async_trait;
use config_keys::{ConfigKey, ConfigParamKey};
use storage::{ConfigRepository, Database, DatabaseError};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn config_value(&self, key: ConfigKey) -> Result<String, DatabaseError>;
    async fn config_param_value(&self, param: ConfigParamKey) -> Result<String, DatabaseError>;
    async fn set_config_value(&self, key: ConfigKey, value: String) -> Result<usize, DatabaseError>;
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
    async fn config_value(&self, key: ConfigKey) -> Result<String, DatabaseError> {
        self.database.run(move |client| client.get_config(key)).await
    }

    async fn config_param_value(&self, param: ConfigParamKey) -> Result<String, DatabaseError> {
        self.database.run(move |client| client.get_config_param(param)).await
    }

    async fn set_config_value(&self, key: ConfigKey, value: String) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.set_config(key, &value)).await
    }
}
