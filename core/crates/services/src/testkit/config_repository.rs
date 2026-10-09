use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use config_keys::{ConfigKey, ConfigParamKey};
use storage::DatabaseError;
use strum::IntoEnumIterator;

use crate::config::repository::Repository;

pub(crate) struct MemoryConfigRepository {
    values: Mutex<HashMap<String, String>>,
    unavailable: bool,
}

impl MemoryConfigRepository {
    pub(crate) fn new() -> Self {
        Self {
            values: Mutex::new(ConfigKey::iter().map(|key| (key.as_ref().to_string(), key.default_value().to_string())).collect()),
            unavailable: false,
        }
    }

    pub(crate) fn unavailable() -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
            unavailable: true,
        }
    }

    pub(crate) fn with_value(self, key: &str, value: &str) -> Self {
        self.values.lock().unwrap().insert(key.to_string(), value.to_string());
        self
    }

    pub(crate) fn without(self, key: &str) -> Self {
        self.values.lock().unwrap().remove(key);
        self
    }

    fn value(&self, key: String) -> Result<String, DatabaseError> {
        if self.unavailable {
            return Err(DatabaseError::ConnectionPool);
        }
        self.values.lock().unwrap().get(&key).cloned().ok_or_else(|| DatabaseError::not_found("Config", key))
    }
}

#[async_trait]
impl Repository for MemoryConfigRepository {
    async fn get_config_value(&self, key: ConfigKey) -> Result<String, DatabaseError> {
        self.value(key.as_ref().to_string())
    }

    async fn get_config_param_value(&self, param: ConfigParamKey) -> Result<String, DatabaseError> {
        self.value(param.key())
    }

    async fn set_config_value(&self, key: ConfigKey, value: String) -> Result<usize, DatabaseError> {
        if self.unavailable {
            return Err(DatabaseError::ConnectionPool);
        }
        self.values.lock().unwrap().insert(key.as_ref().to_string(), value);
        Ok(1)
    }
}
