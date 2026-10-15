use std::time::Duration;

use async_trait::async_trait;
use config_keys::{ConfigKey, ConfigParamKey};
use storage::{ConfigRepository, Database, DatabaseError};

use super::{param_value_or_default, parse_duration};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn get_config_value(&self, key: ConfigKey) -> Result<String, DatabaseError>;
    async fn get_config_param_value(&self, param: ConfigParamKey) -> Result<String, DatabaseError>;
    async fn set_config_value(&self, key: ConfigKey, value: String) -> Result<usize, DatabaseError>;

    async fn get_i64(&self, key: ConfigKey) -> Result<i64, DatabaseError> {
        Ok(self.get_config_value(key).await?.parse()?)
    }

    async fn get_f64(&self, key: ConfigKey) -> Result<f64, DatabaseError> {
        Ok(self.get_config_value(key).await?.parse()?)
    }

    async fn get_duration(&self, key: ConfigKey) -> Result<Duration, DatabaseError> {
        parse_duration(&self.get_config_value(key).await?)
    }

    async fn get_param_bool(&self, param: &ConfigParamKey) -> Result<bool, DatabaseError> {
        Ok(param_value_or_default(self.get_config_param_value(*param).await, param)?.parse()?)
    }
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
    async fn get_config_value(&self, key: ConfigKey) -> Result<String, DatabaseError> {
        self.database.run(move |client| client.get_config(key)).await
    }

    async fn get_config_param_value(&self, param: ConfigParamKey) -> Result<String, DatabaseError> {
        self.database.run(move |client| client.get_config_param(param)).await
    }

    async fn set_config_value(&self, key: ConfigKey, value: String) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.set_config(key, &value)).await
    }
}
